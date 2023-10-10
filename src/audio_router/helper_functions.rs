use std::{
    ffi::{c_void, OsString},
    mem,
    os::windows::prelude::OsStringExt,
    ptr, slice,
};

use image::ImageFormat;
use windows::{
    core::PCWSTR,
    Win32::{
        Devices::Properties,
        Foundation::{GetLastError, HANDLE, HINSTANCE, WIN32_ERROR},
        Graphics::Gdi::{
            DeleteObject, GetBitmapBits, GetObjectW, BITMAP, BITMAPINFOHEADER, HBITMAP, HGDIOBJ,
        },
        Media::Audio::IMMDevice,
        System::{
            Com::StructuredStorage,
            ProcessStatus::{K32GetModuleBaseNameW, K32GetModuleFileNameExW},
        },
        UI::{
            Shell::ExtractIconExW,
            WindowsAndMessaging::{DestroyIcon, GetIconInfoExW, HICON, ICONINFOEXW},
        },
    },
};

pub unsafe fn get_process_name(process: HANDLE) -> Result<String, windows::core::Error> {
    let mut exe_buf = [0u16; 261];
    if K32GetModuleBaseNameW(process, HINSTANCE::default(), &mut exe_buf) > 0 {
        Ok(null_terminated_wchar_to_string(&exe_buf))
    } else {
        Err(GetLastError().unwrap_err())
    }
}

pub unsafe fn null_terminated_wchar_to_string(slice: &[u16]) -> String {
    match slice.iter().position(|&x| x == 0) {
        Some(pos) => OsString::from_wide(&slice[..pos])
            .to_string_lossy()
            .into_owned(),
        None => OsString::from_wide(slice).to_string_lossy().into_owned(),
    }
}

pub fn utf_16_null_terminiated(x: &str) -> Vec<u16> {
    x.encode_utf16().chain(std::iter::once(0)).collect()
}

pub struct ICONHEADER {
    pub id_reserved: i16,
    pub id_type: i16,
    pub id_count: i16,
}

pub struct ICONDIR {
    pub b_width: u8,
    pub b_height: u8,
    pub b_color_count: u8,
    pub b_reserved: u8,
    pub w_planes: u16,    // for cursors, this field = wXHotSpot
    pub w_bit_count: u16, // for cursors, this field = wYHotSpot
    pub dw_bytes_in_res: u32,
    pub dw_image_offset: u32, // file-offset to the start of ICONIMAGE
}

pub fn write_icon_data_to_memory(
    mem: &mut [u8],
    h_bitmap: HBITMAP,
    bmp: &BITMAP,
    bitmap_byte_count: usize,
) {
    unsafe {
        let mut icon_data = Vec::<u8>::with_capacity(bitmap_byte_count);
        icon_data.set_len(bitmap_byte_count);

        GetBitmapBits(
            h_bitmap,
            bitmap_byte_count as i32,
            icon_data.as_mut_ptr() as *mut c_void,
        );

        // bitmaps are stored inverted (vertically) when on disk..
        // so write out each line in turn, starting at the bottom + working
        // towards the top of the bitmap. Also, the bitmaps are stored in packed
        // in memory - scanlines are NOT 32bit aligned, just 1-after-the-other
        let mut pos = 0;
        for i in (0..bmp.bmHeight).rev() {
            // Write the bitmap scanline

            ptr::copy_nonoverlapping(
                icon_data[(i * bmp.bmWidthBytes) as usize..].as_ptr(),
                mem[pos..].as_mut_ptr(),
                bmp.bmWidthBytes as usize,
            ); // 1 line of BYTES
            pos += bmp.bmWidthBytes as usize;

            // extend to a 32bit boundary (in the file) if necessary
            if bmp.bmWidthBytes & 3 != 0 {
                let padding: [u8; 4] = [0; 4];
                ptr::copy_nonoverlapping(
                    padding.as_ptr(),
                    mem[pos..].as_mut_ptr(),
                    (4 - bmp.bmWidthBytes) as usize,
                );
                pos += 4 - bmp.bmWidthBytes as usize;
            }
        }
    }
}

pub unsafe fn get_icon(process: HANDLE) -> Vec<u8> {
    let mut exe_path = [0u16; 128];
    let test = K32GetModuleFileNameExW(process, HINSTANCE::default(), &mut exe_path);
    if test == 0 {
        GetLastError();
    }

    let path = &*null_terminated_wchar_to_string(&exe_path);

    println!("{:?}", null_terminated_wchar_to_string(&exe_path));

    // let test2 = HINSTANCE::default();

    // let instance = GetDriverModuleHandle(HDRVR::default());
    // let mut somthing: [u16; 128] = [0u16; 128];
    // let mut somthing2 = Box::into_raw(Box::new(0u16));

    unsafe fn extract_icon(path: &str, size: i32) -> HICON {
        let mut icon_large = Box::into_raw(Box::new(HICON::default()));
        let mut icon_small = Box::into_raw(Box::new(HICON::default()));

        let extract = ExtractIconExW(
            PCWSTR::from_raw(utf_16_null_terminiated(path).as_ptr()),
            0,
            Some(icon_large),
            Some(icon_small),
            1,
        );
        if extract < 0 {
            GetLastError();
        };
        if size > 16 {
            DestroyIcon(*icon_small);
            *icon_large
        } else {
            DestroyIcon(*icon_large);
            *icon_small
        }
    }

    let mut icon = extract_icon(path, 32);

    // if icon == ptr::null_mut() {
    //     icon = extract_icon("C:\\Windows\\system32\\SHELL32.dll", 32);
    // }

    let icon_info = Box::into_raw(Box::new(ICONINFOEXW::default()));
    let test = GetIconInfoExW(icon, icon_info);
    // println!("{:?}", *icon_info);

    let bmp_color = Box::into_raw(Box::new(BITMAP::default()));
    GetObjectW(
        (*icon_info).hbmColor,
        mem::size_of_val(&bmp_color) as i32,
        Some(bmp_color as *mut c_void),
    );
    // println!("{:?}", *bmp_color);
    let bmp_mask = Box::into_raw(Box::new(BITMAP::default()));
    GetObjectW(
        (*icon_info).hbmMask,
        mem::size_of_val(&bmp_color) as i32,
        Some(bmp_mask as *mut c_void),
    );
    // println!("{:?}", *bmp_mask);

    fn get_bitmap_count(bitmap: &BITMAP) -> i32 {
        let mut n_width_bytes = bitmap.bmWidthBytes;
        // bitmap scanlines MUST be a multiple of 4 bytes when stored
        // inside a bitmap resource, so round up if necessary
        if n_width_bytes & 3 != 0 {
            n_width_bytes = (n_width_bytes + 4) & !3;
        }

        n_width_bytes * bitmap.bmHeight
    }

    let icon_header_size = mem::size_of::<ICONHEADER>();
    let icon_dir_size = mem::size_of::<ICONDIR>();
    let info_header_size = mem::size_of::<BITMAPINFOHEADER>();
    let bitmap_bytes_count = get_bitmap_count(&*bmp_color) as usize;
    let mask_bytes_count = get_bitmap_count(&*bmp_mask) as usize;

    let complete_size =
        icon_header_size + icon_dir_size + info_header_size + bitmap_bytes_count + mask_bytes_count;

    let image_bytes_count = bitmap_bytes_count + mask_bytes_count;
    let mut bytes = Vec::<u8>::with_capacity(complete_size);
    bytes.set_len(complete_size);

    let iconheader = ICONHEADER {
        id_count: 1, // number of ICONDIRs
        id_reserved: 0,
        id_type: 1, // Type 1 = ICON (type 2 = CURSOR)
    };
    let byte_ptr: *mut u8 = mem::transmute(&iconheader);
    ptr::copy_nonoverlapping(byte_ptr, bytes.as_mut_ptr(), icon_header_size);
    let pos = icon_header_size;

    let color_count = if (*bmp_color).bmBitsPixel >= 8 {
        0
    } else {
        1 << ((*bmp_color).bmBitsPixel * (*bmp_color).bmPlanes)
    };

    // Create the ICONDIR structure
    let icon_dir = ICONDIR {
        b_width: (*bmp_color).bmWidth as u8,
        b_height: (*bmp_color).bmHeight as u8,
        b_color_count: color_count,
        b_reserved: 0,
        w_planes: (*bmp_color).bmPlanes,
        w_bit_count: (*bmp_color).bmBitsPixel,
        dw_image_offset: (icon_header_size + 16) as u32,
        dw_bytes_in_res: (mem::size_of::<BITMAPINFOHEADER>() + image_bytes_count) as u32,
    };

    let byte_ptr: *mut u8 = mem::transmute(&icon_dir);
    ptr::copy_nonoverlapping(byte_ptr, bytes[pos..].as_mut_ptr(), icon_dir_size);
    let pos = pos + icon_dir_size;

    let bi_header = BITMAPINFOHEADER {
        biSize: info_header_size as u32,
        biWidth: (*bmp_color).bmWidth,
        biHeight: (*bmp_color).bmHeight * 2, // height of color+mono
        biPlanes: (*bmp_color).bmPlanes,
        biBitCount: (*bmp_color).bmBitsPixel,
        biSizeImage: image_bytes_count as u32,
        biClrImportant: 0,
        biClrUsed: 0,
        biCompression: 0,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0,
    };
    let byte_ptr: *mut u8 = mem::transmute(&bi_header);
    ptr::copy_nonoverlapping(byte_ptr, bytes[pos..].as_mut_ptr(), info_header_size);
    let pos = pos + info_header_size;

    // write the RGBQUAD color table (for 16 and 256 colour icons)
    if (*bmp_color).bmBitsPixel == 2 || (*bmp_color).bmBitsPixel == 8 {}

    write_icon_data_to_memory(
        &mut bytes[pos..],
        (*icon_info).hbmColor,
        &*bmp_color,
        bitmap_bytes_count as usize,
    );
    let pos = pos + bitmap_bytes_count as usize;
    write_icon_data_to_memory(
        &mut bytes[pos..],
        (*icon_info).hbmMask,
        &*bmp_mask,
        mask_bytes_count as usize,
    );

    let im = image::load_from_memory(&bytes).expect("load_from_memory");
    let mut png_bytes: Vec<u8> = Vec::new();
    im.write_to(&mut png_bytes, ImageFormat::Png)
        .expect("write_to");

    DeleteObject(HGDIOBJ::from((*icon_info).hbmColor));
    DeleteObject(HGDIOBJ::from((*icon_info).hbmMask));
    png_bytes
}

pub fn get_hardware_device_name(device: &IMMDevice) -> Result<String, ()> {
    unsafe {
        // Open the device's property store.
        let property_store = device
            .OpenPropertyStore(StructuredStorage::STGM_READ)
            .expect("could not open property store");

        // Get the endpoint's friendly-name property.
        let mut property_value = property_store
            .GetAt(&Properties::DEVPKEY_Device_FriendlyName as *const _ as *const _)
            .unwrap();

        let prop_variant = &property_value.Anonymous.Anonymous;

        // Read the friendly-name from the union data field, expecting a *const u16.
        // if prop_variant.vt != Ole::VT_LPWSTR.0 as _ {

        //     return Err(());
        // }
        let ptr_utf16 = *(&prop_variant.Anonymous as *const _ as *const *const u16);

        // Find the length of the friendly name.
        let mut len = 0;
        while *ptr_utf16.offset(len) != 0 {
            len += 1;
        }

        // Create the utf16 slice and convert it into a string.
        let name_slice = slice::from_raw_parts(ptr_utf16, len as usize);
        let name_os_string: OsString = OsStringExt::from_wide(name_slice);
        let name_string = match name_os_string.into_string() {
            Ok(string) => string,
            Err(os_string) => os_string.to_string_lossy().into(),
        };

        // Clean up the property.
        StructuredStorage::PropVariantClear(&mut property_value).ok();

        Ok(name_string)
    }
}
