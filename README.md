
# Bas Language

*derive from [barelang project](https://github.com/minghu6/barelang)*

## Documents

[Specifications](./spec/)

[Tutorials](./tutorials/)

## Development Setup

*方便起见，目前开发和运行环境是 Linux，但其实是面向所有 POSIX 系统。*

开发工作流由 [mise](https://mise.jdx.dev/) 的任务驱动，`.mise/config.toml` 集中定义环境变量与模板变量，任务定义分散在 `.mise/tasks/*.toml`。

### Setting up Rust nightly

`nightly-2026-08-07`（commit `1a98b1e13`）

### Setting up LLVM-12

如果使用的 Linux 发行版的仓库里恰好有一个 LLVM-12 的包，那么直接安装就是可以，只要确保 `llvm-config` 在 `PATH` 上或者 `LLVM_SYS_120_PREFIX` 环境变量指向 LLVM 的安装位置。

对于 Ubuntu，发行版 20.04-22.04 的仓库有收录，而 [LLVM 官方]( apt.llvm.org)对于 16.04-21.10 也有 deb 安装包。

但对于较新的操作系统，不但没有预编译的二进制包，而且缺乏源代码的编译环境，因为源代码的正确性依赖于特定版本的工具链，而工具链的版本又和发行版绑定，所以这里使用了容器镜像来固化编译环境并用脚本打包运行时，以实现自包含，以这样的方式从源代码开始构建本地 LLVM-12。

>可以直接拉取已经构建好的镜像，通过 mise 任务 `llvmenv:image:pull`。

*容器引擎是 `podman`。*

| Registry | Platforms | Task |
| :--- | :--- | :--- |
| [![GHCR](https://img.shields.io/badge/GHCR-181717?logo=github&logoColor=white)](https://ghcr.io/minghu6/llvm-builder-image) | ![amd64](https://img.shields.io/badge/-amd64-darkgrey?style=flat-square) ![arm64](https://img.shields.io/badge/-arm64-grey?style=flat-square) | `llvmenv:image:pull-ghcr` |
| [![Quay.io](https://img.shields.io/badge/Quay.io-blue?logo=redhat&logoColor=white)](https://quay.io/repository/minghu6/llvm-builder-image) | ![amd64](https://img.shields.io/badge/-amd64-darkgrey?style=flat-square) ![arm64](https://img.shields.io/badge/-arm64-grey?style=flat-square) | `llvmenv:image:pull-quay` |
| [![Huawei Cloud Qingdao](https://img.shields.io/badge/Huawei%20Cloud%20Qingdao-red?logo=huawei&logoColor=white)](https://swr.cn-east-5.myhuaweicloud.com/minghu6/llvm-builder-image) | ![amd64](https://img.shields.io/badge/-amd64-darkgrey?style=flat-square) ![arm64](https://img.shields.io/badge/-arm64-grey?style=flat-square) | `llvmenv:image:pull-hwc-qd` |
| [![Huawei Cloud Shanghai-1](https://img.shields.io/badge/Huawei%20Cloud%20Shanghai--1-red?logo=huawei&logoColor=white)](https://swr.cn-east-3.myhuaweicloud.com/minghu6/llvm-builder-image) | ![amd64](https://img.shields.io/badge/-amd64-darkgrey?style=flat-square) ![arm64](https://img.shields.io/badge/-arm64-grey?style=flat-square) | `llvmenv:image:pull-hwc-sh-1` |
| [![Tencent Cloud](https://img.shields.io/badge/Tencent%20Cloud-006EFF?logo=docker&logoColor=white)](https://ccr.ccs.tencentyun.com/minghu6/llvm-builder-image) | ![amd64](https://img.shields.io/badge/-amd64-darkgrey?style=flat-square) ![arm64](https://img.shields.io/badge/-arm64-grey?style=flat-square) | `llvmenv:image:pull-tc` |
| [![Alibaba Cloud](https://img.shields.io/badge/Alibaba%20Cloud-FF6A00?logo=alibabacloud&logoColor=white)](https://crpi-y7qpip8hchxrp368.cn-hangzhou.personal.cr.aliyuncs.com/minghu6/llvm-builder-image) | ![amd64](https://img.shields.io/badge/-amd64-darkgrey?style=flat-square) ![arm64](https://img.shields.io/badge/-arm64-grey?style=flat-square) | `llvmenv:image:pull-aliyun` |

>或者从 Containerfile 开始构建本地镜像，通过 mise 任务 `llvmenv:image:build-local`。

由于官方源代码存在低级 Bug，因此自己维护了一份儿修复后的克隆仓库 `llvm-project`，通过子模块嵌入本项目；
同时对既有的 Rust 对 LLVM 的高级包装 `inkwell` 进行了方便开发的上游修改，也形成了克隆仓库，通过子模块嵌入本项目。
> 拉取子模块，通过 mise 任务 `gitsubmod:init`。

>使用镜像环境编译并安装 LLVM

| Task | Description |
| :--- | :--- |
| `llvmenv:build-llvm:configure` | Out-of-tree config  |
| `llvmenv:build-llvm:build` | Builds with auto-calculated jobs and memory limits to prevent host swap. |
| `llvmenv:build-llvm:install` | Installs to prefix, bundles runtime libs, and fixes `RPATH` for container independence. |
| `llvmenv:build-llvm:reconfigure` | In-place reconfigure after CMake var changes. |
| `llvmenv:build-llvm:reconfigure-fresh` | Wipes `CMakeCache.txt` and reconfigures from scratch (for toolchain changes). |

- **configure**：链接并发由 `build-resource-config configure` 按总内存给出。
- **build**：`build-resource-config build` 按当前可用内存和 CPU 核数算出 ninja 的 `-j` 与容器 `--memory/--memory-swap`，避免把宿主拖进 SWAP。
- **install**：安装前缀（ `/usr/local/llvm-12`）由 `bundle-runtime-libs` 把 LLVM 二进制依赖的外部共享库（`libz`、`libtinfo`、`libxml2` 等）复制进前缀并改写 `RPATH`，使这个前缀脱离构建容器也能独立使用。

### Running Tests

| Task | Description |
| :--- | :--- |
| `bas:test:codegen` | Runs codegen test with `--nocapture`. |
| `bas:test:boot` | Runs boot test with `RUST_BACKTRACE=1`. |
| `bas:test:spec` | Runs spec test with `--nocapture`. |

## Demos

[Demo files](./examples/)

| Task | Description |
| :--- | :--- |
| `basc:verify:exp0` | Compiles `exp0`, then runs `./exp0`. |
| `basc:verify:test_sepc` | Compiles the `test_spec` example (check failure output). |

## Follow-Up

### Design

其实在开发的过程中，最大的问题是语言设计的问题，如果事先没有想好一个完整的语言设计，而是一边实现一边设计，那会是非常痛苦，从最前端词法语法解析到语义分析再到LLVM-IR的生成，相当于要把整个项目改一遍，这种做法实在不可取！

特别是，我并没有特别想要做的语言设计，充其量有一些语法上的想法，项目也是为了好玩儿和做技术验证，之所以设定为一个脚本目的的语言也是为了不让它变得很复杂、庞大，能有一个现实的用途，以现实使用的反馈指导设计。

并且传统的 shell 语言都很不体面，以 Bash 为例，居然是直接在Token流上做语义分析，而传统的通用脚本语言，Perl的严格模式已经把它的真正优势自我阉割了，相对最好用的还得是Python ，但总是割开着一层正规语言的壳子，而最好的脚本语言一定是简单直接的，好像在 shell 里写代码一样，就像 Perl 曾经的那样。

由于这两方面的原因，因此成为了 Bas 初始的设计目标，但是具体设计还并不完整，强行固化也可以，但是和传统语言一样也就没意思了。

### To-do List

虽然还没有完整具体的语言设计，但是还是有很多既有的工作可以来做， 包括不限于如下：

1. 完成算术除法，新手任务，因为绝大部分工作已经做完了，只需要照着乘法实现，在语义分析和LLVM-IR生成阶段配置下代码即可；
2. 词法分析出的错误适当处理，稍微进阶一点的任务，由于编译器首先是作者自己用，所以不会犯词法上的错误，这导致有关报错信息一直没有适当处理；
3. 函数的定义与调用机制的测试和调通（main 函数的定义和调用已经过了，其他函数问题也不大），中级任务；
4. 基于函数名解析的抽象接口的自动实现，复杂的高级任务；
   4.1 提供对抽象接口的特定类型实现的检查
5. 引用外部代码模块的机制补全，高级任务；
6. 实现一个合适的机制，包括对源文件的无注解的语法树的序列化，和缓存既有语法树的编译文件，使得可以作为脚本语言方便运行，高级任务；
7. 增加一套 IO 的标准库函数，便于完成基本的文本处理工作， 高级任务；
8. 按照 DWARF 格式在 LLVM 生成代码时嵌入 Debug 信息，繁琐的高级任务；
9. `cmd` 依赖 POSIX exec 实现，绑定了 `sh`，可以改进为灵活支持 `bash` 等更多 shell，繁琐的中级任务；
