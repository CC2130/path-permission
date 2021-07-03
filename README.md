# path-permission
rust Path(Buf) 获取其权限（permission）的特性（trait)
在已有一个确定的路径时，可依此库来获取路径文件的权限。
它为`Path(Buf)`提供了一个名为`PathPermission`的特性（trait），在使用`Path`类型
时，可以此获取其文件权限，是否可读（r）、可写（w）、可执行（x）。  
目前只支持 *Unix* 类系统。

## 示例
```rust
use std::path::Path;

extern crate path_permission;

use path_permission::*;

let path = Path::new("src/lib.rs");

assert_eq!(path.is_readable().unwrap(), true);
assert_eq!(path.is_writable().unwrap(), true);
assert_eq!(path.is_excutable().unwrap(), false);
assert_eq!(path.is_removable().unwrap(), true);

let new_path = Path::new("a/b/d/e/f");

assert_eq!(new_path.is_creatable().unwrap(), true);
```
  
## 注意
在使用时需注意：  
  * 当返回值为Ok(false)时，意为无查看此路径的权限，即可能此路径不存在（从起始
路径开始，至无访问权限的子级路径，是存在的）。  
  * 如无必须，不太建议使用相对路径，可使用[path-calculate](https://crates.io/
crates/path-calculate)将路径转换为绝对路径。  
  
## 后续计划
将完善，使之支持使用位掩码（bitmask）`764`，或`rwx`类型查看与转换。  
暂不支持粘滞位的判断处理。  
  
## 感谢
此项目部分代码，来源自项目[permissions](https://crates.io/crates/permissions)。  
