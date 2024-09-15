
# Bas Lanuage

*derive from [barelang project](https://github.com/minghu6/barelang)*

## Documents

[Specifications](./spec/)

[Tutorials](./tutorials/)

## Dependencies

*方便起见，目前开发和运行环境是Linux，但其实是面向所有POSIX系统*

### LLVM-12

如果使用的 Linux 发行版的仓库里恰好有一个 LLVM-12 的包，那么直接安装就是一个选择，但如果没有，或者想要更加控制地编译想要的版本，那么就需要从源代码开始编译。

#### Build & Install

首先获取 LLVM-12 的源代码，但是该死的 LLVM 仓库里对应v12的代码居然有非常低级的Bug，于是我fork了源代码仓库，并做了修改。

`git clone --single-branch -b release/12.x.m6.fix --depth 1 https://github.com/minghu6/llvm-project-m6.git llvm-project`

构建过程和详细参数配置可以参考[clang子项目的building介绍](https://clang.llvm.org/get_started.html)，写得比 LLVM 本家项目要好。

**进入构建目录**

```bash
cd llvm-project
mkdir build # (in-tree build is not supported)
cd build
```

#### 生成构建配置

有几个需要关注的配置项（[详细参考](https://llvm.org/docs/CMake.html#llvm-related-variables)）：

**构建类型**

`CMAKE_BUILD_TYPE = Debug | RelWithDebInfo | Release | MinSizeRel`

Debug：如果既需要 Debug LLVM生成的代码，又需要 Debug LLVM 本身
RelWithDebInfo： 仅需要 Debug LLVM生成的代码
Release：只使用
MinSizeRel：只使用，而且减少空间占用

**编译器和链接器**

`CMAKE_CXX_COMPILER = clang++ | g++ (default)`

`DCMAKE_C_COMPILER = clang | gcc (default)`

`DLLVM_USE_LINKER = lld | ld (default)`

clang 项目生成代码的优化水平比 gcc 项目还是强不少的，特别是使用 lld ，可以使 LLVM 的构建速度加快不少。

**构建系统**

`G: Ninja | Unix Makefiles | Visual Studio | Xcode`

对本项目的 Linux 来说实际就是 Ninja 和 Unix Makefiles

Ninja 构建更快，但是要小心配置，否则它会吃掉你机器的所有资源，一家伙把你的用户会话挂掉。(可以参考[这里](https://github.com/ninja-build/ninja/issues/2187)和[这里](https://github.com/ninja-build/ninja/issues/1441))

*from abstract to real*

```bash
cmake -DCMAKE_POLICY_DEFAULT_CMP0116=OLD \
    -DCMAKE_CXX_FLAGS="-Wno-deprecated -Wno-bitwise-instead-of-logical -Wno-deprecated-copy" \
    -DCMAKE_CXX_STANDARD=17 \
    -DCMAKE_BUILD_TYPE=RelWithDebInfo \
    -DCMAKE_CXX_COMPILER=clang++ \
    -DCMAKE_C_COMPILER=clang \
    -DLLVM_USE_LINKER=lld \
    -G "Ninja" \
    ../llvm
```

如果要更换工具链，那么需要 cmake 前面传入一个 `--fresh` 参数，来移除所有的 `CMakeCache.txt` ，但这也意味着要完全重新进行编译，所以开始工具链的选择要慎重。

#### 构建

这里不通过 cmake 而直接调用构建工具是为了方便配置参数，特别是如果前面没用 `LLVM_PARALLEL_{COMPILE,LINK,TABLEGEN}_JOBS` 指定并发数量。

个人经验，有几个编译和链接任务存在内存瓶颈，可能需要手动降低下并发数，进而降低内存消耗，避免内存不足或者进入SWAP，特别是如果编译得是 Debug 版本。

另外注意，ninja 是默认并发 6 ，而 make 默认不并发。

```bash
ninja -j3  # cmake --build .
```

#### 安装

假设目录安装在 `/usr/local/llvm-12` 上。

```bash
sudo cmake -DCMAKE_INSTALL_PREFIX=/usr/local/llvm-12 -P cmake_install.cmake
```

### Configure

最体面的方式应该是提前考虑对多个版本的 LLVM 的管理问题，但是没有一个成熟又普适的现成管理工具，于是考虑最小化代价的解决方案，通过管理环境变量的方式做版本管理。

可以开发一个工具，就像是 pyenv，rbenv 做的那样，但为了节约时间和精力，直接试用下现成的基于目录配置环境变量的工具 direnv 。

#### [direnv](https://direnv.net/)

在我们的项目目录或者某个合适层级的父目录下创建我们的环境配置文件 `.envrc`

```bash
# LLVM
export LLVM_ROOT="/usr/local/llvm-12"
export PATH="$LLVM_ROOT/bin:$PATH"
export LLVM_SYS_120_PREFIX=$LLVM_ROOT
```
对于我们项目来说，只需要配置 `LLVM_SYS_120_PREFIX` 或者 包含 `llvm_config` 的 `PATH` 即可，全都配置只是便于它用。

但是这里要提一个恼人的问题，direnv 每次加载配置文件都会有丑陋的强制提示，官方配置也不能彻底取消，可以在 shell session 的初始化文件里紧接着 direnv hook 之后覆盖相关函数。

```bash
eval "$(direnv hook bash)"

copy_function() {
    test -n "$(declare -f "$1")" || return
    eval "${_/$1/$2}"
}

copy_function _direnv_hook _direnv_hook__old

_direnv_hook() {
    _direnv_hook__old "$@" 2> /dev/null
}
```
`_direnv_hook` 是 direnv 内部 hardcode 的 shell 函数。

并且（后来发现）由于 direnv 劫持了 shell ，导致其他基于路径（`PATH`）插值的版本管理工具无法正常工作，因此只能限制 direnv 自动加载的范围。

`~/.config/direnv/direnv.toml`

```Toml
[global]
# https://direnv.net/man/direnv.toml.1.html
hide_env_diff = true

[whitelist]
prefix=["~/coding/Rust"]
```
这里就把目录限制在 `~/coding/Rust` 目录下面。

其实理论上 direnv 的天才想法是用户每次手动 `direnv allow` 去允许加载一个目录的 `.envrc`，然后用 `direnv disallow`（它还有两个别名叫做 `deny` 和 `revoke`） 去撤回对环境变量的加载，可与其这样，那还不如我们自己写一个保存和恢复环境变量的脚本，我们需要得是自动静默加载，就像其他版本管理工具做的那样，所以这里直接把目录加到白名单里。


*如果在使用`direnv`的过程中，发现它到处都很怪很难用，不用担心，这是正常现象，因为它是一个用Go写的用户程序。Go的哲学就是“当下能凑活用就行”，也是一种美国后现代工程文化的代表。如果在一个多语言项目里其他项目都成功了，只有一个语言的项目失败了，那它就是Go的项目，比如对编译好的LLVM运行测试，大概会发现只有Go的测试失败了，不用在意，也不必浪费时间寻找原因，完全就是Go自己的用户接口的健壮性问题。*

### Co-operation Projects

#### inkwel-m6

对官方 [inkwell](https://github.com/TheDan64/inkwell) 的 fork 版本，方便控制版本变化。
它是对上游 [llvm-sys](https://crates.io/crates/llvm-sys) 的高级抽象， llvm-sys 通过指定环境变量 `LLVM_SYS_<VERSION>_PREFIX` 或者调用 `PATH` 上的 `llvm-config` ，直接绑定具体的 LLVM（的 C API）。

`git clone https://github.com/minghu6/m6inkwell.git`

#### inkwellkit

对 `inkwell` 的工具包，主要是提供了一些预处理宏，方便做代码 binding 。


`git clone https://github.com/minghu6/inkwellkit.git`

#### lexkit

词法分析工具包

`git clone https://github.com/minghu6/m6lexerkit.git`

#### parserkit

语法分析工具包

`git clone https://github.com/minghu6/m6parserkit.git`


## Demos

[Demo files](./examples/)

`make testexp0`

*编译器的运行不依赖 LLVM ，代价是较大的空间占用，不过这对现代通用型编译器来说是科学的 Trade Off，进一步的讨论在[这里]()。*

## Follow-Up

### 语言设计

其实在开发的过程中，最大的问题是语言设计的问题，如果事先没有想好一个完整的语言设计，而是一边实现一边设计，那会是非常痛苦，从最前端词法语法解析到语义分析再到LLVM-IR的生成，相当于要把整个项目改一遍，这种做法实在不可取！

特别是，我并没有特别想要做的语言设计，充其量有一些语法上的想法，项目也是为了好玩儿和做技术验证，之所以设定为一个脚本目的的语言也是为了不让它变得很复杂、庞大，能有一个现实的用途，以现实使用的反馈指导设计。

并且传统的 shell 语言都很不体面，以 Bash 为例，居然是直接在Token流上做语义分析，而传统的通用脚本语言，Perl的严格模式已经把它的真正优势自我阉割了，相对最好用的还得是Python ，但总是搁着一层正规语言的壳子，而最好的脚本语言一定是简单直接的，好像在 shell 里写代码一样，就像 Perl 曾经的那样。

由于这两方面的原因，因此成为了 Bas 初始的设计目标，但是具体设计还并不完整，强行固化也可以，但是和传统语言一样也就没意思了。

### 既有补完

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

