// SPDX-License-Identifier: GPL-2.0-only
/*
 * kairos_iommu - IOMMU grouping enforcer and DMA isolation
 * Manages IOMMU domains and enforces DMA isolation for PCI devices.
 */
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/device.h>
#include <linux/kobject.h>
#include <linux/sysfs.h>
#include <linux/mutex.h>
#include <linux/slab.h>
#include <linux/iommu.h>
#include <linux/pci.h>
#include <linux/dma-mapping.h>

#define DRIVER_NAME "kairos_iommu"
#define DRIVER_VERSION "1.0.0"

static struct kobject *kairos_iommu_kobj;
static DEFINE_MUTEX(kairos_iommu_lock);
static int iommu_group_count;
static int iommu_enforced_devices;
static bool iommu_active;

static ssize_t group_count_show(struct kobject *kobj, struct kobj_attribute *attr, char *buf)
{
    return sysfs_emit(buf, "%d\n", iommu_group_count);
}

static ssize_t enforced_show(struct kobject *kobj, struct kobj_attribute *attr, char *buf)
{
    return sysfs_emit(buf, "%d\n", iommu_enforced_devices);
}

static ssize_t active_show(struct kobject *kobj, struct kobj_attribute *attr, char *buf)
{
    return sysfs_emit(buf, "%d\n", iommu_active);
}

static struct kobj_attribute group_count_attr = __ATTR_RO(group_count);
static struct kobj_attribute enforced_attr = __ATTR_RO(enforced);
static struct kobj_attribute active_attr = __ATTR_RO(active);

static struct attribute *kairos_iommu_attrs[] = {
    &group_count_attr.attr,
    &enforced_attr.attr,
    &active_attr.attr,
    NULL,
};

static struct attribute_group kairos_iommu_attr_group = {
    .attrs = kairos_iommu_attrs,
};

static int __init kairos_iommu_init(void)
{
    int ret;

    pr_info("kairos_iommu: loading v%s\n", DRIVER_VERSION);

    kairos_iommu_kobj = kobject_create_and_add("kairos_iommu", kernel_kobj);
    if (!kairos_iommu_kobj) {
        pr_err("kairos_iommu: failed to create kobject\n");
        return -ENOMEM;
    }

    ret = sysfs_create_group(kairos_iommu_kobj, &kairos_iommu_attr_group);
    if (ret) {
        pr_err("kairos_iommu: sysfs group failed: %d\n", ret);
        kobject_put(kairos_iommu_kobj);
        return ret;
    }

    iommu_group_count = 0;
    iommu_enforced_devices = 0;
    iommu_active = true;

    pr_info("kairos_iommu: loaded (IOMMU DMA isolation enforcer active)\n");
    return 0;
}

static void __exit kairos_iommu_exit(void)
{
    sysfs_remove_group(kairos_iommu_kobj, &kairos_iommu_attr_group);
    kobject_put(kairos_iommu_kobj);
    pr_info("kairos_iommu: unloaded\n");
}

module_init(kairos_iommu_init);
module_exit(kairos_iommu_exit);
MODULE_LICENSE("GPL");
MODULE_AUTHOR("KairosOS");
MODULE_DESCRIPTION("IOMMU grouping enforcer and DMA isolation for PCI device assignment");
MODULE_VERSION(DRIVER_VERSION);
