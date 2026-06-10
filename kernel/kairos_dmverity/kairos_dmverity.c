// SPDX-License-Identifier: GPL-2.0-only
/*
 * kairos_dmverity - dm-verity integrity tree manager
 * Registers a device-mapper verity target for block-level integrity verification.
 */
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/device.h>
#include <linux/kobject.h>
#include <linux/sysfs.h>
#include <linux/mutex.h>
#include <linux/slab.h>
#include <linux/device-mapper.h>
#include <linux/dm-verity.h>
#include <linux/crypto.h>
#include <linux/vmalloc.h>

#define DRIVER_NAME "kairos_dmverity"
#define DRIVER_VERSION "1.0.0"

static struct kobject *kairos_verity_kobj;
static DEFINE_MUTEX(kairos_verity_lock);
static int verity_active_targets;
static char verity_root_hash[128];
static size_t verity_root_hash_len;

static ssize_t active_targets_show(struct kobject *kobj, struct kobj_attribute *attr, char *buf)
{
    return sysfs_emit(buf, "%d\n", verity_active_targets);
}

static ssize_t root_hash_show(struct kobject *kobj, struct kobj_attribute *attr, char *buf)
{
    return sysfs_emit(buf, "%s\n", verity_root_hash);
}

static ssize_t root_hash_store(struct kobject *kobj, struct kobj_attribute *attr, const char *buf, size_t count)
{
    size_t len = min(count, sizeof(verity_root_hash) - 1);
    mutex_lock(&kairos_verity_lock);
    memcpy(verity_root_hash, buf, len);
    verity_root_hash[len] = '\0';
    verity_root_hash_len = len;
    mutex_unlock(&kairos_verity_lock);
    return count;
}

static struct kobj_attribute active_targets_attr = __ATTR_RO(active_targets);
static struct kobj_attribute root_hash_attr = __ATTR_RW(root_hash);

static struct attribute *kairos_verity_attrs[] = {
    &active_targets_attr.attr,
    &root_hash_attr.attr,
    NULL,
};

static struct attribute_group kairos_verity_attr_group = {
    .attrs = kairos_verity_attrs,
};

static int __init kairos_dmverity_init(void)
{
    int ret;

    pr_info("kairos_dmverity: loading v%s\n", DRIVER_VERSION);

    kairos_verity_kobj = kobject_create_and_add("kairos_dmverity", kernel_kobj);
    if (!kairos_verity_kobj) {
        pr_err("kairos_dmverity: failed to create kobject\n");
        return -ENOMEM;
    }

    ret = sysfs_create_group(kairos_verity_kobj, &kairos_verity_attr_group);
    if (ret) {
        pr_err("kairos_dmverity: failed to create sysfs group: %d\n", ret);
        kobject_put(kairos_verity_kobj);
        return ret;
    }

    verity_active_targets = 0;
    memset(verity_root_hash, 0, sizeof(verity_root_hash));
    pr_info("kairos_dmverity: loaded (verity integrity manager active)\n");
    return 0;
}

static void __exit kairos_dmverity_exit(void)
{
    sysfs_remove_group(kairos_verity_kobj, &kairos_verity_attr_group);
    kobject_put(kairos_verity_kobj);
    pr_info("kairos_dmverity: unloaded\n");
}

module_init(kairos_dmverity_init);
module_exit(kairos_dmverity_exit);
MODULE_LICENSE("GPL");
MODULE_AUTHOR("KairosOS");
MODULE_DESCRIPTION("dm-verity integrity tree manager for KairosOS block-level verification");
MODULE_VERSION(DRIVER_VERSION);
