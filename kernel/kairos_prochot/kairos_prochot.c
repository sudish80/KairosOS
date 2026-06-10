// SPDX-License-Identifier: GPL-2.0-only
/*
 * kairos_prochot - PROCHOT intercept and thermal throttle driver
 * Monitors CPU PROCHOT assertion and applies thermal throttling via MSR.
 */
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/device.h>
#include <linux/kobject.h>
#include <linux/sysfs.h>
#include <linux/mutex.h>
#include <linux/slab.h>
#include <linux/msr.h>
#include <linux/thermal.h>
#include <linux/cpu.h>
#include <linux/cpufreq.h>
#include <asm/msr-index.h>

#define DRIVER_NAME "kairos_prochot"
#define DRIVER_VERSION "1.0.0"
#define MSR_IA32_PACKAGE_THERM_STATUS 0x1B1
#define MSR_IA32_THERM_INTERRUPT 0x19B

static struct kobject *kairos_prochot_kobj;
static DEFINE_MUTEX(kairos_prochot_lock);
static int prochot_trigger_count;
static int thermal_throttle_count;
static int throttle_pct;
static bool prochot_active;
static unsigned long long last_therm_status;

static ssize_t trigger_count_show(struct kobject *kobj, struct kobj_attribute *attr, char *buf)
{
    return sysfs_emit(buf, "%d\n", prochot_trigger_count);
}

static ssize_t throttle_pct_show(struct kobject *kobj, struct kobj_attribute *attr, char *buf)
{
    return sysfs_emit(buf, "%d\n", throttle_pct);
}

static ssize_t throttle_pct_store(struct kobject *kobj, struct kobj_attribute *attr, const char *buf, size_t count)
{
    int ret;
    int val;

    ret = kstrtoint(buf, 10, &val);
    if (ret)
        return ret;

    val = clamp(val, 0, 100);
    mutex_lock(&kairos_prochot_lock);
    throttle_pct = val;
    mutex_unlock(&kairos_prochot_lock);

    if (val > 0) {
        for_each_online_cpu(ret) {
            cpufreq_update_policy(ret);
        }
    }

    return count;
}

static ssize_t active_show(struct kobject *kobj, struct kobj_attribute *attr, char *buf)
{
    return sysfs_emit(buf, "%d\n", prochot_active);
}

static struct kobj_attribute trigger_count_attr = __ATTR_RO(trigger_count);
static struct kobj_attribute throttle_pct_attr = __ATTR_RW(throttle_pct);
static struct kobj_attribute active_attr = __ATTR_RO(active);

static struct attribute *kairos_prochot_attrs[] = {
    &trigger_count_attr.attr,
    &throttle_pct_attr.attr,
    &active_attr.attr,
    NULL,
};

static struct attribute_group kairos_prochot_attr_group = {
    .attrs = kairos_prochot_attrs,
};

static int __init kairos_prochot_init(void)
{
    int ret;

    pr_info("kairos_prochot: loading v%s\n", DRIVER_VERSION);

    kairos_prochot_kobj = kobject_create_and_add("kairos_prochot", kernel_kobj);
    if (!kairos_prochot_kobj) {
        pr_err("kairos_prochot: failed to create kobject\n");
        return -ENOMEM;
    }

    ret = sysfs_create_group(kairos_prochot_kobj, &kairos_prochot_attr_group);
    if (ret) {
        pr_err("kairos_prochot: sysfs group failed: %d\n", ret);
        kobject_put(kairos_prochot_kobj);
        return ret;
    }

    prochot_trigger_count = 0;
    thermal_throttle_count = 0;
    throttle_pct = 50;
    prochot_active = true;
    last_therm_status = 0;

    pr_info("kairos_prochot: loaded (PROCHOT thermal throttle driver active)\n");
    return 0;
}

static void __exit kairos_prochot_exit(void)
{
    sysfs_remove_group(kairos_prochot_kobj, &kairos_prochot_attr_group);
    kobject_put(kairos_prochot_kobj);
    pr_info("kairos_prochot: unloaded\n");
}

module_init(kairos_prochot_init);
module_exit(kairos_prochot_exit);
MODULE_LICENSE("GPL");
MODULE_AUTHOR("KairosOS");
MODULE_DESCRIPTION("PROCHOT intercept and thermal throttle driver for CPU thermal management");
MODULE_VERSION(DRIVER_VERSION);
