use std::{
    io,
    path::{Path, PathBuf},
    os::{
        raw::c_int,
        unix::ffi::OsStrExt,
    },
};

pub trait PathPermission {
    /// 判断路径是否可读
    fn is_readable(&self) -> io::Result<bool>;

    /// 判断路径是否可写
    fn is_writable(&self) -> io::Result<bool>;

    /// 判断路径是否可执行
    fn is_excutable(&self) -> io::Result<bool>;

    /// 判断路径可否被创建（当前无此路径）
    fn is_creatable(&self) -> io::Result<bool>;
}

impl PathPermission for Path {
    fn is_readable(&self) -> io::Result<bool> {
        access(self, libc::R_OK)
    }

    fn is_writable(&self) -> io::Result<bool> {
        access(self, libc::W_OK)
    }

    fn is_excutable(&self) -> io::Result<bool> {
        access(self, libc::X_OK)
    }

    fn is_creatable(&self) -> io::Result<bool> {
        let parent = match self.parent() {
            // 此时已无父级目录，则此路径为相对路径，其起始位置为当前目录。
            // 不建议使用相对路径
            None => Path::new("./"),
            Some(parent) => parent,
        };
        if ! parent.exists() {
            parent.is_creatable()
        } else {
            // parent 一定存在，可直接使用unwrap()获取结果
            if parent.is_writable().unwrap() & parent.is_excutable().unwrap() {
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
}

impl PathPermission for PathBuf {
    fn is_readable(&self) -> io::Result<bool> {
        self.as_path().is_readable()
    }

    fn is_writable(&self) -> io::Result<bool> {
        self.as_path().is_writable()
    }

    fn is_excutable(&self) -> io::Result<bool> {
        self.as_path().is_excutable()
    }

    fn is_creatable(&self) -> io::Result<bool> {
        self.as_path().is_creatable()
    }
}

fn access(path: &Path, mod_mask: c_int) ->io::Result<bool> {
    let mut buf = Vec::new();
    let buf_ptr;

    // 在C中，char的最后一位是'\0'或ASCII码值为0
    buf.extend(path.as_os_str().as_bytes());
    buf.push(0);

    buf_ptr = buf.as_ptr() as *const libc::c_char;

    let permission = unsafe {
        libc::access(buf_ptr, mod_mask)
    };

    match permission {
        0 => Ok(true),
        _ => {
            let err = io::Error::last_os_error();
            if err.raw_os_error().unwrap() == libc::EACCES {
                Ok(false)  // 无查看此路径的权限（无法确认路径是否存在）
            } else {
                Err(err)  // 其它错误，如路径不存在等
            }
        }
    }
}
