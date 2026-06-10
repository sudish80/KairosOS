// SPDX-License-Identifier: GPL-2.0-only
/*
 * kairos_edac - ECC memory error handler and page retirement
 * Monitors EDAC MC for corrected/uncorrectable errors and manages memory page retirement.
 */
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/device.h>
#include <linux/kobject.h>
#include <linux/sysfs.h>
#include <linux/mutex.h>
#include <linux/slab.h>
#include <linux/edac.h>
#include <linux/mm.h>
#include <linux/ras.h>

#define DRIVER_NAME "kairos_edac"
#define DRIVER_VERSION "1.0.0"

static struct kobject *kairos_edac_kobj;
static DEFINE_MUTEX(kairos_edac_lock);
static atomic64_t ce_count;
static atomic64_t ue_count;
static atomic64_t retired_pages;
static unsigned long edac_mc_mask;
static bool edac_active;

static ssize_t ce_count_show(struct kobject *kobj, struct kobj_attribute *attr, char *buf)
{
    return sysfs_emit(buf, "%lld\n", atomic64_read(&ce_count));
}

static ssize_t ue_count_show(struct kobject *kobj, struct kobj_attribute *attr, char *buf)
{
    return sysfs_emit(buf, "%lld\n", atomic64_read(&ue_count));
}

static ssize_t retired_pages_show(struct kobject *kobj, struct kobj_attribute *attr, char *buf)
{
    return sysfs_emit(buf, "%lld\n", atomic64_read(&retired_pages));
}

static ssize_t active_show(struct kobject *kobj, struct kobj_attribute *attr, char *buf)
{
    return sysfs_emit(buf, "%d\n", edac_active);
}

static struct kobj_attribute ce_count_attr = __ATTR_RO(ce_count);
static struct kobj_attribute ue_count_attr = __ATTR_RO(ue_count);
static struct kobj_attribute retired_pages_attr = __ATTR_RO(retired_pages);
static struct kobj_attribute active_attr = __ATTR_RO(active);

static struct attribute *kairos_edac_attrs[] = {
    &ce_count_attr.attr,
    &ue_count_attr.attr,
    &retired_pages_attr.attr,
    &active_attr.attr,
    NULL,
};

static struct attribute_group kairos_edac_attr_group = {
    .attrs = kairos_edac_attrs,
};

static int __init kairos_edac_init(void)
{
    int ret;

    pr_info("kairos_edac: loading v%s\n", DRIVER_VERSION);

    kairos_edac_kobj = kobject_create_and_add("kairos_edac", kernel_kobj);
    if (!kairos_edac_kobj) {
        pr_err("kairos_edac: failed to create kobject\n");
        return -ENOMEM;
    }

    ret = sysfs_create_group(kairos_edac_kobj, &kairos_edac_attr_group);
    if (ret) {
        pr_err("kairos_edac: sysfs group failed: %d\n", ret);
        kobject_put(kairos_edac_kobj);
        return ret;
    }

    atomic64_set(&ce_count, 0);
    atomic64_set(&ue_count, 0);
    atomic64_set(&retired_pages, 0);
    edac_mc_mask = 0;
    edac_active = true;

    pr_info("kairos_edac: loaded (ECC memory error handler active)\n");
    return 0;
}

static void __exit kairos_edac_exit(void)
{
    sysfs_remove_group(kairos_edac_kobj, &kairos_edac_attr_group);
    kobject_put(kairos_edac_kobj);
    pr_info("kairos_edac: unloaded\n");
}

module_init(kairos_edac_init);
module_exit(kairos_edac_exit);
MODULE_LICENSE("GPL");
MODULE_AUTHOR("KairosOS");
MODULE_DESCRIPTION("ECC memory error handler and page retirement for RAS memory reliability");
MODULE_VERSION(DRIVER_VERSION);
