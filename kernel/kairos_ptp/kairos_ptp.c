// SPDX-License-Identifier: GPL-2.0-only
/*
 * kairos_ptp - Precision Time Protocol hardware sync
 * Registers a PTP clock for hardware timestamping and synchronizes system time.
 */
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/device.h>
#include <linux/kobject.h>
#include <linux/sysfs.h>
#include <linux/mutex.h>
#include <linux/slab.h>
#include <linux/ptp_clock_kernel.h>
#include <linux/net_tstamp.h>
#include <linux/timekeeper.h>
#include <linux/timex.h>
#include <linux/err.h>

#define DRIVER_NAME "kairos_ptp"
#define DRIVER_VERSION "1.0.0"

static struct kobject *kairos_ptp_kobj;
static DEFINE_MUTEX(kairos_ptp_lock);
static struct ptp_clock *kairos_ptp_clock;
static struct ptp_clock_info kairos_ptp_clock_info;
static int ptp_adjfreq_ppb;
static u64 ptp_tsc_freq;

static int kairos_ptp_adjfine(struct ptp_clock_info *ptp, long scaled_ppm)
{
    mutex_lock(&kairos_ptp_lock);
    ptp_adjfreq_ppb = scaled_ppm;
    mutex_unlock(&kairos_ptp_lock);
    return 0;
}

static int kairos_ptp_gettime(struct ptp_clock_info *ptp, struct timespec64 *ts)
{
    *ts = ktime_to_timespec64(ktime_get_real());
    return 0;
}

static int kairos_ptp_settime(struct ptp_clock_info *ptp, const struct timespec64 *ts)
{
    do_settimeofday64(ts);
    return 0;
}

static int kairos_ptp_enable(struct ptp_clock_info *ptp, struct ptp_clock_request *rq, int on)
{
    return -EOPNOTSUPP;
}

static ssize_t frequency_show(struct kobject *kobj, struct kobj_attribute *attr, char *buf)
{
    return sysfs_emit(buf, "%d\n", ptp_adjfreq_ppb);
}

static ssize_t clock_name_show(struct kobject *kobj, struct kobj_attribute *attr, char *buf)
{
    return sysfs_emit(buf, "kairos_ptp\n");
}

static struct kobj_attribute frequency_attr = __ATTR_RO(frequency);
static struct kobj_attribute clock_name_attr = __ATTR_RO(clock_name);

static struct attribute *kairos_ptp_attrs[] = {
    &frequency_attr.attr,
    &clock_name_attr.attr,
    NULL,
};

static struct attribute_group kairos_ptp_attr_group = {
    .attrs = kairos_ptp_attrs,
};

static int __init kairos_ptp_init(void)
{
    int ret;

    pr_info("kairos_ptp: loading v%s\n", DRIVER_VERSION);

    kairos_ptp_kobj = kobject_create_and_add("kairos_ptp", kernel_kobj);
    if (!kairos_ptp_kobj) {
        pr_err("kairos_ptp: failed to create kobject\n");
        return -ENOMEM;
    }

    ret = sysfs_create_group(kairos_ptp_kobj, &kairos_ptp_attr_group);
    if (ret) {
        pr_err("kairos_ptp: sysfs group failed: %d\n", ret);
        kobject_put(kairos_ptp_kobj);
        return ret;
    }

    kairos_ptp_clock_info.owner = THIS_MODULE;
    kairos_ptp_clock_info.name = "kairos_ptp";
    kairos_ptp_clock_info.max_adj = 500000000;
    kairos_ptp_clock_info.n_alarm = 0;
    kairos_ptp_clock_info.n_ext_ts = 0;
    kairos_ptp_clock_info.n_per_out = 0;
    kairos_ptp_clock_info.n_pins = 0;
    kairos_ptp_clock_info.pps = 0;
    kairos_ptp_clock_info.adjfine = kairos_ptp_adjfine;
    kairos_ptp_clock_info.gettime64 = kairos_ptp_gettime;
    kairos_ptp_clock_info.settime64 = kairos_ptp_settime;
    kairos_ptp_clock_info.enable = kairos_ptp_enable;

    kairos_ptp_clock = ptp_clock_register(&kairos_ptp_clock_info, NULL);
    if (IS_ERR(kairos_ptp_clock)) {
        pr_err("kairos_ptp: ptp_clock_register failed: %ld\n", PTR_ERR(kairos_ptp_clock));
        sysfs_remove_group(kairos_ptp_kobj, &kairos_ptp_attr_group);
        kobject_put(kairos_ptp_kobj);
        return PTR_ERR(kairos_ptp_clock);
    }

    ptp_adjfreq_ppb = 0;
    ptp_tsc_freq = 0;
    pr_info("kairos_ptp: loaded (PTP hardware clock registered)\n");
    return 0;
}

static void __exit kairos_ptp_exit(void)
{
    if (kairos_ptp_clock && !IS_ERR(kairos_ptp_clock))
        ptp_clock_unregister(kairos_ptp_clock);
    sysfs_remove_group(kairos_ptp_kobj, &kairos_ptp_attr_group);
    kobject_put(kairos_ptp_kobj);
    pr_info("kairos_ptp: unloaded\n");
}

module_init(kairos_ptp_init);
module_exit(kairos_ptp_exit);
MODULE_LICENSE("GPL");
MODULE_AUTHOR("KairosOS");
MODULE_DESCRIPTION("Precision Time Protocol hardware sync driver for network timestamping");
MODULE_VERSION(DRIVER_VERSION);
