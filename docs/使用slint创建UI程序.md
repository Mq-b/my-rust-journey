# 使用 `slint` 创建 UI 程序

[`slint`](https://github.com/slint-ui/slint) 是一个用于创建图形界面的开源框架。

使用 slint 创建 UI 程序通常遵循“设计与逻辑解耦”的模式。无论你使用何种语言，UI 界面统一在 `.slint` 文件中定义。slint 编译器会将这些定义翻译为对应的 Rust、C++ 或 JavaScript 代码，并利用各语言的原生工具链进行构建，从而实现跨平台的高效开发。

这种设计事实上非常常见，现代的 UI 框架大多开始采用类似做法。且需要注意，这不是套壳浏览器。

## 安装配置 `slint`

在 `Cargo.toml` 中，我们需要分别在 `[dependencies]` 和 `[build-dependencies]` 中配置 Slint：

```toml
[build-dependencies]
slint-build = "1.0.0"       # 构建时依赖：用于在编译阶段处理 .slint 文件

[dependencies]
slint = "1.0.0"				# 运行时依赖：提供运行所需的库函数和组件
```

> [!NOTE]
> 项目编译开始时，`build-dependencies` 中的 `slint-build` 会通过 `build.rs` 脚本运行。它负责将声明式的 `.slint` 界面文件转换为 Rust 源代码。
>
> “*运行时依赖*”的措辞实则不够恰当，因为完全可以是静态链接 `slint`，使用 slint UI 框架构建的程序并不需要额外打包什么即可直接发布。这里其实强调的是 slint 作为一个库本身，我们的项目使用到了它，自然 rust 编译的时候需要能找到它，理所应当。

同时在 `build.rs` 中添加需要使用的 `.slint` UI 文件：

```rust
fn main() {
    slint_build::compile("ui/barcode.slint").unwrap();
}
```

> [!NOTE]
> `ui/barcode.slint` 是 slint 文件的路径，表示需要使用 slint 编译的 UI 文件。slint 的编译工具会将其编译成 Rust 代码，并生成相应的模块供 Rust 程序使用。

## 引入 `.slint` 文件到 Rust 源代码

通过使用宏 `slint::include_modules!();` 可以将 `.slint` 文件编译生成的 Rust 模块引入到 Rust 源代码中。

例如先前我们的 `build.rs` 中编译了 `ui/barcode.slint`，则在 Rust 代码中可以这样引入：

```rust
slint::include_modules!();

fn main(){
    // BarcodeWindow 是由 ui/barcode.slint 编译生成的 barcode.rs 模块中的组件
    let window = BarcodeWindow::new().unwrap(); 
}
```

实际上是包含了将 `ui/barcode.slint` 编译生成的 `barcode.rs` 模块引入到了当前的 Rust 源代码中。

## 关闭控制台

默认来说，启动 rust 构建的 UI 程序会打开一个控制台窗口，这对于UI程序没什么意义，我们可以在你的主入口点，如 `main.rs` 的顶部添加：

```rust
#![windows_subsystem = "windows"]
```

则不会再弹出控制台窗口了。

## 设置程序图标

软件图标分为两种：

1. 运行时图标
2. 可执行程序图标

运行时图标我们可以通过 `slint` 的 `window` 组件的 `icon` 属性来设置：

```slint
export component AppWindow inherits Window {
    // 自动将图片资源编译进二进制文件
    icon: @image-url("../assets/icon.svg");
    // ... 其他属性
}
```

而可执行程序图标在 `Windows` 则需要通过传统 `.rc` 文件来设置，且要求图标的格式必须为 `.ico`。

创建 `icon.rc` 文件：

```rc
iconName ICON "./assets/icon.ico"
```

路径可自行修改。然后在 `build.rs` 中添加：

```rust
embed_resource::compile("./icon.rc");
```

图标的设置是加入到可执行程序本身，所以发布程序时无需将图标文件一起打包。

## 💡 友情提示

### 1. 关于编译机制与缓存

Rust 的 `cargo` 构建系统采用的是**全源码编译**模式。在首次构建或引入新的三方依赖时，`cargo` 会拉取所有依赖的源代码并在本地进行编译。

**建议**：除非遇到难以解决的编译错误，否则**请勿轻易清理构建缓存**（如执行 `cargo clean`），以避免漫长的二次编译等待。

### 2. 关于 `target` 目录体积

由于包含了中间编译产物、调试信息和静态链接的库文件，即使是一个中小型项目，其生成的 `target` 目录体积也可能迅速膨胀（甚至达到 20GB 甚至更高）。

**Git 配置**：在项目根目录的 `.gitignore` 文件中添加 `target/` 记录，防止将巨大的构建产物提交到 Git 仓库中。
