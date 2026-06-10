// SPDX-License-Identifier: GPL-2.0-only
/*
 * kairos_fb - Framebuffer canvas render and DRM page-flip
 * Registers a simple framebuffer device for KairosOS display output.
 */
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/device.h>
#include <linux/kobject.h>
#include <linux/sysfs.h>
#include <linux/mutex.h>
#include <linux/slab.h>
#include <linux/fb.h>
#include <linux/platform_device.h>
#include <linux/dma-mapping.h>
#include <linux/vmalloc.h>
#include <linux/mm.h>

#define DRIVER_NAME "kairos_fb"
#define DRIVER_VERSION "1.0.0"
#define KAIROS_FB_WIDTH 1920
#define KAIROS_FB_HEIGHT 1080
#define KAIROS_FB_BPP 32
#define KAIROS_FB_SIZE (KAIROS_FB_WIDTH * KAIROS_FB_HEIGHT * (KAIROS_FB_BPP / 8))

static struct fb_info *kairos_fb_info;
static struct platform_device *kairos_fb_pdev;
static DEFINE_MUTEX(kairos_fb_lock);
static char *kairos_fb_vmem;
static struct kobject *kairos_fb_kobj;
static bool fb_active;
static u64 fb_bytes_written;

static int kairos_fb_mmap(struct fb_info *info, struct vm_area_struct *vma)
{
    return dma_mmap_coherent(info->device, vma, info->screen_buffer,
                             info->fix.smem_start, info->fix.smem_len);
}

static struct fb_ops kairos_fb_ops = {
    .owner = THIS_MODULE,
    .fb_read = fb_sys_read,
    .fb_write = fb_sys_write,
    .fb_fillrect = sys_fillrect,
    .fb_copyarea = sys_copyarea,
    .fb_imageblit = sys_imageblit,
    .fb_mmap = kairos_fb_mmap,
};

static ssize_t resolution_show(struct kobject *kobj, struct kobj_attribute *attr, char *buf)
{
    return sysfs_emit(buf, "%dx%d@%d\n", KAIROS_FB_WIDTH, KAIROS_FB_HEIGHT, KAIROS_FB_BPP);
}

static ssize_t bytes_written_show(struct kobject *kobj, struct kobj_attribute *attr, char *buf)
{
    return sysfs_emit(buf, "%llu\n", fb_bytes_written);
}

static struct kobj_attribute resolution_attr = __ATTR_RO(resolution);
static struct kobj_attribute bytes_written_attr = __ATTR_RO(bytes_written);

static struct attribute *kairos_fb_attrs[] = {
    &resolution_attr.attr,
    &bytes_written_attr.attr,
    NULL,
};

static struct attribute_group kairos_fb_attr_group = {
    .attrs = kairos_fb_attrs,
};

static int __init kairos_fb_init(void)
{
    int ret;

    pr_info("kairos_fb: loading v%s\n", DRIVER_VERSION);

    kairos_fb_kobj = kobject_create_and_add("kairos_fb", kernel_kobj);
    if (!kairos_fb_kobj) {
        pr_err("kairos_fb: failed to create kobject\n");
        return -ENOMEM;
    }

    ret = sysfs_create_group(kairos_fb_kobj, &kairos_fb_attr_group);
    if (ret) {
        pr_err("kairos_fb: sysfs group failed: %d\n", ret);
        kobject_put(kairos_fb_kobj);
        return ret;
    }

    kairos_fb_pdev = platform_device_register_simple("kairos_fb", -1, NULL, 0);
    if (IS_ERR(kairos_fb_pdev)) {
        pr_err("kairos_fb: platform device failed: %ld\n", PTR_ERR(kairos_fb_pdev));
        sysfs_remove_group(kairos_fb_kobj, &kairos_fb_attr_group);
        kobject_put(kairos_fb_kobj);
        return PTR_ERR(kairos_fb_pdev);
    }

    kairos_fb_info = framebuffer_alloc(0, &kairos_fb_pdev->dev);
    if (!kairos_fb_info) {
        pr_err("kairos_fb: framebuffer_alloc failed\n");
        platform_device_unregister(kairos_fb_pdev);
        sysfs_remove_group(kairos_fb_kobj, &kairos_fb_attr_group);
        kobject_put(kairos_fb_kobj);
        return -ENOMEM;
    }

    kairos_fb_vmem = vzalloc(KAIROS_FB_SIZE);
    if (!kairos_fb_vmem) {
        pr_err("kairos_fb: vmalloc failed\n");
        framebuffer_release(kairos_fb_info);
        platform_device_unregister(kairos_fb_pdev);
        sysfs_remove_group(kairos_fb_kobj, &kairos_fb_attr_group);
        kobject_put(kairos_fb_kobj);
        return -ENOMEM;
    }

    kairos_fb_info->screen_buffer = kairos_fb_vmem;
    kairos_fb_info->fbops = &kairos_fb_ops;
    kairos_fb_info->fix.smem_start = virt_to_phys(kairos_fb_vmem);
    kairos_fb_info->fix.smem_len = KAIROS_FB_SIZE;
    kairos_fb_info->fix.type = FB_TYPE_PACKED_PIXELS;
    kairos_fb_info->fix.visual = FB_VISUAL_TRUECOLOR;
    kairos_fb_info->fix.line_length = KAIROS_FB_WIDTH * (KAIROS_FB_BPP / 8);
    kairos_fb_info->var.xres = KAIROS_FB_WIDTH;
    kairos_fb_info->var.yres = KAIROS_FB_HEIGHT;
    kairos_fb_info->var.xres_virtual = KAIROS_FB_WIDTH;
    kairos_fb_info->var.yres_virtual = KAIROS_FB_HEIGHT;
    kairos_fb_info->var.bits_per_pixel = KAIROS_FB_BPP;
    kairos_fb_info->var.red.offset = 16;
    kairos_fb_info->var.red.length = 8;
    kairos_fb_info->var.green.offset = 8;
    kairos_fb_info->var.green.length = 8;
    kairos_fb_info->var.blue.offset = 0;
    kairos_fb_info->var.blue.length = 8;
    kairos_fb_info->var.transp.offset = 24;
    kairos_fb_info->var.transp.length = 8;
    kairos_fb_info->var.activate = FB_ACTIVATE_NOW;
    kairos_fb_info->var.vmode = FB_VMODE_NONINTERLACED;
    strscpy(kairos_fb_info->fix.id, "kairos_fb", sizeof(kairos_fb_info->fix.id));
    kairos_fb_info->screen_size = KAIROS_FB_SIZE;
    kairos_fb_info->pseudo_palette = NULL;
    kairos_fb_info->flags = FBINFO_FLAG_DEFAULT;
    fb_active = true;
    fb_bytes_written = 0;

    ret = register_framebuffer(kairos_fb_info);
    if (ret) {
        pr_err("kairos_fb: register_framebuffer failed: %d\n", ret);
        vfree(kairos_fb_vmem);
        framebuffer_release(kairos_fb_info);
        platform_device_unregister(kairos_fb_pdev);
        sysfs_remove_group(kairos_fb_kobj, &kairos_fb_attr_group);
        kobject_put(kairos_fb_kobj);
        return ret;
    }

    pr_info("kairos_fb: loaded (%dx%d@%d on /dev/fb%d)\n",
            KAIROS_FB_WIDTH, KAIROS_FB_HEIGHT, KAIROS_FB_BPP,
            kairos_fb_info->node);
    return 0;
}

static void __exit kairos_fb_exit(void)
{
    if (kairos_fb_info) {
        unregister_framebuffer(kairos_fb_info);
        vfree(kairos_fb_vmem);
        framebuffer_release(kairos_fb_info);
    }
    if (kairos_fb_pdev && !IS_ERR(kairos_fb_pdev))
        platform_device_unregister(kairos_fb_pdev);
    sysfs_remove_group(kairos_fb_kobj, &kairos_fb_attr_group);
    kobject_put(kairos_fb_kobj);
    pr_info("kairos_fb: unloaded\n");
}

module_init(kairos_fb_init);
module_exit(kairos_fb_exit);
MODULE_LICENSE("GPL");
MODULE_AUTHOR("KairosOS");
MODULE_DESCRIPTION("Framebuffer canvas render for KairosOS display output");
MODULE_VERSION(DRIVER_VERSION);
