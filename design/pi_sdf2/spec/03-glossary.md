# 词汇表

> 定义“是什么”，不定义“做什么”。每条 _Avoid_ 行列出被否决的同义词。
> 中英对应显式给出，供接口与目录命名统一使用。

## 几何

### TERM-001 轮廓 Outline

字形或输入路径的完整边界几何，由一个或多个子轮廓组成，以弧与线段表示。

_Avoid_: 边界、外框、contour（指整体时）、路径集合

中英：Outline

关系：Outline 1—N Contour

### TERM-002 子轮廓 Contour

轮廓中一条闭合的边界环；其绕行方向（顺 / 逆时针）用于判定内外。

_Avoid_: 环、闭合路径、loop、闭合段

中英：Contour

关系：Contour 1—N Arc

### TERM-003 弧 Arc

由两个端点与一个曲率参数（记作 d，等于四分之一圆心角的正切）确定的圆弧；d 为 0 时退化为线段。

_Avoid_: 圆弧、曲线、circle、segment（作泛指时）

中英：Arc

### TERM-004 弧端点 ArcEndpoint

弧的可传递表示：位置坐标加曲率参数，用于跨模块与跨语言边界传输几何。

_Avoid_: 端点、控制点、point（指传输形式时）

中英：ArcEndpoint

### TERM-005 包围盒 Aabb

与坐标轴对齐的矩形范围，用于格元划分、纹理布局与边界判定。

_Avoid_: bbox、外框、区域、rect

中英：Aabb

## SDF 与光栅

### TERM-006 有符号距离场 SDF

对平面上每一点给出到轮廓最短距离的标量场，符号表示点在轮廓内还是轮廓外。

_Avoid_: 距离场、有向距离、signed distance

中英：SDF / Signed Distance Field

### TERM-007 格元 Cell

递归细分产生的一个矩形区域，是 SDF 计算的独立单位。

_Avoid_: 网格、单元、tile、块、区域

中英：Cell

### TERM-008 近邻弧 Near Arcs

某个格元的 SDF 计算可能用到的弧集合，即距该格元足够近的弧。

_Avoid_: 邻近弧、候选弧、相关弧

中英：Near Arcs

### TERM-009 近邻弧网格 Cell Grid

由全部格元及其近邻弧索引构成的整体结构，是轮廓与 SDF 纹理之间的中间态。

_Avoid_: 四叉树、网格、cell 列表、空间索引

中英：Cell Grid

### TERM-010 SDF 纹理 SDF Texture

光栅化后的 SDF 数据，按纹理尺寸的二维网格存储，供渲染端采样。

_Avoid_: 距离图、bitmap、sdf 图片

中英：SDF Texture

### TERM-011 距离像素范围 PxRange

SDF 纹理中数值 0 到 1 所覆盖的像素距离范围，决定边缘过渡的宽度。

_Avoid_: 范围、range、扩展量、border

中英：PxRange

### TERM-012 纹理尺寸 TexSize

SDF 纹理的边长（本库使用方形纹理）。

_Avoid_: 尺寸、分辨率、size、宽高

中英：TexSize

### TERM-013 量化 Quantize

把浮点坐标或距离映射为指定位宽的整数表示。

_Avoid_: 压缩、离散化、取整

中英：Quantize

### TERM-014 烘焙 Bake

把近邻弧网格转换为 SDF 纹理相关数据的过程。

_Avoid_: 编码、编译、生成纹理

中英：Bake

### TERM-015 数据纹理 Data Texture

烘焙产物之一，存放量化后的弧端点编码。

_Avoid_: 端点纹理、arc 纹理

中英：Data Texture

### TERM-016 索引纹理 Index Texture

烘焙产物之一，存放格元到数据纹理的映射与距离区间。

_Avoid_: 映射纹理、offset 纹理

中英：Index Texture

### TERM-017 单位弧 Unit Arc

烘焙时归属某个格元的一条弧记录，含弧端点与距离区间。

_Avoid_: 弧条目、item、entry

中英：Unit Arc

## 字体与文本

### TERM-018 字形 Glyph

字体中一个字符对应的形状。

_Avoid_: 字符、字、符号、font char

中英：Glyph

### TERM-019 字形步进 Advance

排布文本时，一个字形之后光标前进的水平距离。

_Avoid_: 宽度、间距、advance width、步长

中英：Advance

### TERM-020 字体单位 UnitsPerEm

字体设计坐标系的每 em 单位数，用于设计坐标与像素坐标之间的换算。

_Avoid_: em、upem、字号、scale

中英：UnitsPerEm

## 平台与接口

### TERM-021 平台后端 Platform Backend

承载平台相关职责（字体来源、字节编解码、内存分配器）的适配层。

_Avoid_: 适配器、平台层、platform 层

中英：Platform Backend

### TERM-022 算法核心 Algorithm Core

与平台无关的计算部分：几何、轮廓、SDF、烘焙、光栅化。

_Avoid_: 内核、core、算法层（作名词时）

中英：Algorithm Core

### TERM-023 边界壳 Boundary Shell

平台后端中负责参数编解码、随后委托算法核心的薄层。

_Avoid_: 包装、wrapper、导出层、胶水

中英：Boundary Shell

## 通用词的处置

几何与工程通用词（Point、Vector、SignedVector、Line、Segment、Bezier、Direction、Result、Error）不属于项目特有概念，按本技能 D3 的「只收项目特有概念」规则**不单列词条**；其含义由 Rust 标准语义与几何常识给定。签名中出现这些词不视为词汇表缺口。

### TERM-024 ArcArena 弧数据索引 arena

以索引而非裸指针共享单位弧的容器。

_Avoid_: 弧池、指针表、arena 缓存

中英：ArcArena

### TERM-025 FontFace 字体面

一个已解析字体及其轮廓提取能力的持有者。

_Avoid_: 字体对象、font、字形集

中英：FontFace

### TERM-026 GlyphMetrics 字形度量

单字形的步进、包围盒与绕向。

_Avoid_: 字形信息、度量表、font metrics

中英：GlyphMetrics

### TERM-027 Path 路径

由路径动词序列与坐标序列描述的矢量路径。

_Avoid_: 折线、polyline、svg 对象

中英：Path

### TERM-028 PathVerb 路径动词

路径中一段动作的类型，判别值与 JS u8 契约一致（1..=19）。

_Avoid_: 命令、command、路径指令

中英：PathVerb

### TERM-029 Shape 图形元

圆、矩形、线段、椭圆、多边形、折线的统称。

_Avoid_: 图元、primitive、几何体

中英：Shape

### TERM-030 SvgScene SVG 场景

若干图形元与一个视口构成的集合。

_Avoid_: 画布、场景树、scene graph

中英：SvgScene

### TERM-031 FontSource 字体来源

字体的取得途径：文件路径或系统字体族。

_Avoid_: 字体加载器、font loader、字体集合

中英：FontSource

### TERM-032 RasterOptions 光栅化选项

光栅化时影响输出形态的开关（外发光、SVG、内外反转）。

_Avoid_: 参数、配置、options

中英：RasterOptions

### TERM-033 TextureInfo 纹理定位信息

供渲染端定位 SDF 纹理的平面边界、图集边界与偏移。

_Avoid_: 纹理元数据、uv 信息

中英：TextureInfo

### TERM-034 TextureLayout 纹理布局

由轮廓范围与参数算出的平面边界、图集边界、距离与纹理尺寸。

_Avoid_: 排版、布局表、layout 结果

中英：TextureLayout

### TERM-035 StrokeMesh 描边网格

描边生成的位置、纹理坐标与三角形索引三元组。

_Avoid_: 描边顶点、mesh、几何网格

中英：StrokeMesh

## 自检

- [x] 每个术语都有 TERM-NNN，写在标题开头，且都将回填 00-index 的「词汇」表
- [x] 未引用其他设计术语（Glob 只有本 slug，术语引用表留空）
- [x] 每条定义说的是“是什么”，未出现职责 / 行为描述
- [x] 每条都有 _Avoid_ 行，且至少列了一个被否决的同义词
- [x] 收的都是项目特有概念，未包含 User / Config / Logger 等通用词
- [x] 无“可选 A 或 B”表述
- [x] 中英对应关系已写明
- [x] 与 D1 需求名词一致（轮廓 / 近邻弧 / SDF 纹理 / 平台），无被否决叫法需要替换
- [x] 超过 15 条，已按 4 个领域分组
- [x] 已确认无需 Read 其他设计词汇表（无其他设计）
- [x] 无跨设计同词不同义冲突
