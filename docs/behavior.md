# 行为说明

## 分词

AC 自动机 + 三种切分算法（`segmentit`）：

- `ReverseMaxMatch`（1）：逆向最大匹配，最快、准确率适中。
- `MaxProbability`（2，默认）：最大概率，准确率高。
- `MinTokenization`（3）：最少分词数。

内置词库：单字 2 万余、双字词 2150+、三字 323、四字 1593（含成语）、
姓氏 492、数字规则。完整版词库（34.8 万词）需自行 `add_dict` 载入。

一个实现细节（与 JS 逐行一致）：`add_dict` / `custom_pinyin`
每次都会为新增节点重建 fail 指针，但从不改动老节点的 fail 指向。
增量词是否能参与匹配，取决于这一点，不要想当然。

## 一 / 不变调

- `一` 后跟四声 → 二声（`一个` → `yí gè`），后跟一/二/三声 → 四声。
- `不` 后跟四声 → 二声（`不要` → `bú yào`）。
- 以下字后不变调：一/不 + `的|而|之|后|也|还`（一还有 `是`，不还有 `地`）。
- 叠词轻声：`说一说` → `shuō yi shuō`。
- `tone_sandhi: false` 把结果中的一/不恢复为 `yī` / `bù`。

## 特殊单字

- `了`：前字非汉字（含开头）时读 `liǎo`。
- `々`：跟读前字首读音；无前字或前字非汉字时读 `tóng`（`展々` → `zhǎn zhǎn`）。

## 姓氏

`mode=Surname` 等价于 `surname=All`。`surname=Head` 只对文本开头用姓氏读音
（`曾乐乐` → `zēng lè lè`）。单字姓氏（如 `查`→`zhā`、`区`→`ōu`、`朴`→`piáo`）
优先级高于普通单字读音。`multiple` 下姓氏读音排首位（`能` → `nài néng`）。

## 多音与 `type=all`

`multiple=true` 仅对单字输入生效，返回全部读音并去重。
`type=all` 的 `polyphonic` 字段是全部读音（当前读音排首位），
`num` 取自原读音的声调。

## 非汉字

`non_zh=Spaced`（默认）逐字保留；`Consecutive` 把相连的非汉字合并为一段
（`ab汉语` → `ab hàn yǔ`）；`Removed` / `remove_non_zh=true` 移除。
`non_zh_scope` 是正则字符串，只有命中的字符才参与上述处理。

## 繁体

先用 `add_traditional_dict` 登记繁→简映射，再对输入开 `traditional=true`。
映射只影响查词，返回的原字仍是输入的繁体字。

## 全局状态

`custom_pinyin`、`add_dict`、`add_traditional_dict` 都是进程全局
（内部 `RwLock`，线程安全）。测试或长进程里用完记得
`clear_custom_dict` / `remove_dict` 清理，否则互相污染。

## 与原 JS 版的差异

- `match` 返回字符序号：纯 BMP 文本与 JS 的 UTF-16 序号一致；非 BMP 字符
  计 1 位（JS 计 2 位，如 `𧒽`）。
- `non_zh_scope` 传正则字符串（JS 传 `RegExp`）。
- 类型系统代替了运行时的 `validateType`：传错类型编译期就拦下了；
  `polyphonic("")` 返回空，`pinyin("")` 按 `type_mode` 返回空。
- 其余行为（含分词 tie-break、空位补空串、非法选项值的“静默无视”语义）
  与 JS 逐分支对齐，有 4221 个 differential 用例锁定。
