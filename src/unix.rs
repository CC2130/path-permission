use std::{
    io,
    path::{Path, PathBuf},
    os::{
        raw::c_int,
        unix::{
            ffi::OsStrExt,
            fs::MetadataExt,
        }
    },
};

pub trait PathPermission {
    /// 检查对路径的权限，通过1(x)、2(w)、4(r)
    fn access(&self, amode: c_int) -> io::Result<bool>;

    /// 判断路径是否可读
    fn is_readable(&self) -> io::Result<bool>;

    /// 判断路径是否可写
    fn is_writable(&self) -> io::Result<bool>;

    /// 判断路径是否可执行
    fn is_excutable(&self) -> io::Result<bool>;

    /// 判断路径可否被创建（当前无此路径）
    fn is_creatable(&self) -> io::Result<bool>;

    /// 判断路径能否被删除
    fn is_removable(&self) -> io::Result<bool>;

    /// 检查文件的权限
    /// mode 可习惯上使用8进制数字，如：0o0644
    /// The file type and mode: The stat.st_mode contains the file type and mode.
    /// 帮助手册[inode(7)](https://man7.org/linux/man-pages/man7/inode.7.html)
    fn check_access(&self, mode: u16) -> io::Result<bool>;

    /// 返回路径的权限，以 stat 的形式：0o0644
    /// 注意：已经格式化为字符串！
    fn get_access(&self) -> io::Result<String>;

    /// 变更文件的权限
    /// mode 可习惯上使用8进制数字，如：0o0644
    fn chmod(&self, mode: u16) -> io::Result<bool>;
}

impl PathPermission for Path {
    fn access(&self, amode: c_int) -> io::Result<bool> {
        access(self, amode)
    }

    fn is_readable(&self) -> io::Result<bool> {
        self.access(libc::R_OK)
    }

    fn is_writable(&self) -> io::Result<bool> {
        self.access(libc::W_OK)
    }

    fn is_excutable(&self) -> io::Result<bool> {
        self.access(libc::X_OK)
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
            // 需要对父级目录有写和读的权限（1 + 2 = 3)
            parent.access(libc::X_OK + libc::W_OK)
        }
    }

    fn is_removable(&self) -> io::Result<bool> {
        // 文件不存在时，返回Ok(false)
        if ! self.exists() {
            return Ok(false)
        }
        let parent = match self.parent() {
            None => Path::new("./"),
            Some(parent) => parent,
        };

        // 如果父级目录没有设置 S_ISVTX
        // 需要对父级目录有写和读的权限（1 + 2 = 3)
        if ! parent.check_access(0o1000).unwrap() {
            parent.access(libc::X_OK + libc::W_OK)
        } else {
            // 需进行是否为本用户所属文件判断
            unsafe {
                if libc::getuid() == self.metadata().unwrap().uid() {
                    parent.access(libc::X_OK + libc::W_OK)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn check_access(&self, mode: u16) -> io::Result<bool> {
        if let Ok(metadata) = self.metadata() {
            if metadata.mode() as u16 & mode == mode {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(io::Error::last_os_error())
        }
    }

    fn get_access(&self) -> io::Result<String> {
        if let Ok(metadata) = self.metadata() {
            Ok(format!("{:o}{:o}",
                       metadata.mode() as u16 & 0o7000,
                       metadata.mode() as u16 & 0o777))
        } else {
            Err(io::Error::last_os_error())
        }
    }

    fn chmod(&self, mode: u16) -> io::Result<bool> {
        chmod(self, mode)
    }
}

impl PathPermission for PathBuf {
    fn access(&self, amode: c_int) -> io::Result<bool> {
        self.as_path().access(amode)
    }

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

    fn is_removable(&self) -> io::Result<bool> {
        self.as_path().is_removable()
    }

    fn check_access(&self, mode: u16) -> io::Result<bool> {
        self.as_path().check_access(mode)
    }

    fn get_access(&self) -> io::Result<String> {
        self.as_path().get_access()
    }

    fn chmod(&self, mode: u16) -> io::Result<bool> {
        self.as_path().chmod(mode)
    }
}

fn access(path: &Path, mod_mask: c_int) ->io::Result<bool> {
    let mut buf = Vec::new();
    let buf_ptr;

    // 在C中，char的最后一位是'\0'或ASCII码值为0
    buf.extend(path.as_os_str().as_bytes());
    buf.push(0);

    buf_ptr = buf.as_ptr() as *const libc::c_char;

    let result = unsafe {
        libc::access(buf_ptr, mod_mask)
    };

    match result {
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

fn chmod(path: &Path, mode: u16) -> io::Result<bool> {
    let mut buf = Vec::new();
    let buf_ptr;

    // 在C中，char的最后一位是'\0'或ASCII码值为0
    buf.extend(path.as_os_str().as_bytes());
    buf.push(0);

    buf_ptr = buf.as_ptr() as *const libc::c_char;

    let result = unsafe {
        libc::chmod(buf_ptr, mode)
    };

    match result {
        0 => Ok(true),
        // 1: PermissionDenied, 2: No such file or directory
        1 | 2 => Ok(false),
        _ => Err(io::Error::last_os_error()),
    }
}
