# ECL `!ins_signatures` 缩写推断

这份文档基于两类材料做谨慎推断：

- [`tools/th20.eclm`](../tools/th20.eclm) 里的 `!ins_names` 和 `!ins_signatures`
- [`tools/st01.decl`](../tools/st01.decl) 里的真实反编译调用形式

目标不是宣称“完全准确”，而是为 THTK-Studio 的参数模板、hover 和补全提供一份当前可用的解释层。

## 当前推断表

| 缩写 | 当前推断 | 置信度 | 依据 |
| --- | --- | --- | --- |
| `f` | `float` | 高 | `enemy_set_hitbox(ff)`、`move_velocity_abs(ff)`、`move_orbit_rel(ffff)` 与反编译结果完全吻合 |
| `S` | `int` / 整数槽位 | 高 | `ecl_time_sub(S)`、`enemy_life_set(S)`、`anm_set_slot(SS)`、`set_int_difficulty(SSSSS)` |
| `m` | `subroutine` / 子程序引用 | 中高 | `call(m*D)`、`async_call(m*D)`、`timer_callback_sub(Sm)`、`callback_ex(SSSm)` |
| `D` | `label` / 跳转目标 | 中 | `call(m*D)`、`async_call(m*D)` 中和控制流最匹配，推测是标签/地址目标 |
| `o` | `offset` / 相对跳转位置 | 中 | `jump(ot)`、`jump_equ(ot)`、`jump_neq(ot)`；与 `goto Label @ 0` 的双参数形式接近 |
| `t` | `time` / 帧延迟 | 中 | `jump(ot)` 的第二个参数很像 `@ 0` 这种时间参数 |
| `x` | `difficulty` / 难度掩码 | 中高 | `spellcard_start(SSSx)`、`spellcard_start_difficulty(SSSx)` 与难度开关最匹配 |
| `*` | `ref` / 指针式输出或引用修饰 | 中 | 出现在 `m*D`、`lookup_int(SSS*D)`、`lookup_float(fSf*D)`，像“引用结果槽”或“输出参数”修饰 |

## 逐项说明

### `f`

这个目前最明确。

典型对应：

- `enemy_set_hitbox(24.0f, 24.0f)` 对应 `500 ff`
- `enemy_set_collision(16.0f, 16.0f)` 对应 `501 ff`
- `move_velocity_abs(angle, speed)` 对应 `404 ff`
- `move_velocity_abs_interp(60, 1, angle, speed)` 对应 `405 SSff`

因此 `f = float` 基本可以视为确定。

### `S`

也比较明确，表示整数型参数。

典型对应：

- `ecl_time_sub(120)` 对应 `23 S`
- `enemy_life_set(...)` 对应 `511 S`
- `anm_set_slot(slot, script)` 对应 `303 SS`
- `set_int_difficulty(...)` 对应 `529 SSSSS`

这里的 `S` 有时可能是布尔、枚举、ID、flag，但底层都可先视为“整数类槽位”。

### `m`

目前最合理的推断是“子程序引用”。

依据：

- `11 call` 对应签名 `m*D`
- `15 async_call` 对应签名 `m*D`
- `521 timer_callback_sub` 对应签名 `Sm`
- `514 callback_ex` 对应签名 `SSSm`

这些名字都强烈暗示会传入一个脚本子程序。

### `D`

当前更像“标签”或“跳转目标”。

它总是出现在控制流相关的 `call / async_call / lookup_*` 等签名里，尤其 `m*D` 这类组合，像是“调用目标 + 返回/跳转位置”。

这里不能 100% 下结论，但对于 IDE 当前用途，先把它当 `label` 是合理的。

### `o`

当前推断为“offset”或“跳转位置参数”。

依据：

- `12 jump`
- `13 jump_equ`
- `14 jump_neq`

这些都对应 `ot`。结合反编译里大量出现的：

```ecl
goto GirlA01_448 @ 0;
```

最保守解释是：

- `o` 表示跳转目标位置
- `t` 表示附带的时间/偏移参数

### `t`

当前更像“时间”或“帧偏移”。

它在 `jump(ot)` 这种位置最典型，对应反编译语法里的 `@ 0` 很合理。

所以目前把它解释为 `time` 是比较稳妥的。

### `x`

当前推断为“难度掩码/难度参数”。

依据：

- `522 spellcard_start` -> `SSSx`
- `528 spellcard_start_2` -> `SSSx`
- `531/532/533 spellcard_start_difficulty*` -> `SSSx`

这些名字本身就和难度选择强相关，因此 `x = difficulty` 的可信度比较高。

### `*`

`*` 本身不像独立参数，更像修饰符。

当前观察：

- `call -> m*D`
- `lookup_int -> SSS*D`
- `lookup_float -> fSf*D`

这更像是“后面的参数是引用型/输出型/地址型参数”。对 IDE 当前模板生成来说，先把它解释成 `ref_` 前缀是合理的。

## 结合 `st01.decl` 的进一步印证

真实反编译代码里这些模式能进一步支持上面的推断：

- `move_velocity_abs(angle, speed)` 明确印证 `ff`
- `move_velocity_abs_interp(60, 1, angle, speed)` 明确印证 `SSff`
- `move_orbit_rel(%RAND_ANGLE, 0.017453292f, 32.0f, 0.0f)` 明确印证 `ffff`
- `enemy_set_hitbox(24.0f, 24.0f)` 和 `enemy_set_collision(16.0f, 16.0f)` 印证 `ff`
- `ecl_time_sub(120)`、`ecl_time_sub([-1])` 印证 `S`
- `goto Label @ 0` 支持 `ot` 中“目标 + 时间/偏移”的推断

## 对 IDE 的实际用途

基于这份推断，当前 THTK-Studio 已经可以先做这些事情：

1. 用 `f / S / m / D / o / t / x` 生成更像样的参数模板
2. 在 hover 里显示签名原文和推断后的参数类型
3. 在补全里用泛型参数名：
   - `float1`
   - `int1`
   - `subroutine1`
   - `label1`
   - `offset1`
   - `time1`
   - `difficulty1`

## 当前边界

这份推断仍然有明确边界：

- 目前还不能保证所有签名字符都已完全理解
- `*D` 的精确语义仍不确定，当前只适合当“引用型标签/目标参数”处理
- 这还不足以宣称已经拥有完整 ECL 语义系统

因此后续实现应继续保持：

- 参数模板可以利用这些推断
- hover 可以展示“推断类型”
- 诊断和更深层语义不要过度自信

## 推荐后续

下一步最值得做的是：

1. 在 hover 里同时显示：
   - 原始签名
   - 推断后的参数列表
2. 为最常用指令补一层人工命名参数表
3. 用更多反编译样本继续验证 `m / D / o / t / x / *` 的含义

## 待办

- [ ] 建立“常用指令人工参数名映射表”，优先覆盖高频指令：
  - `move_velocity_abs`
  - `move_velocity_abs_interp`
  - `move_orbit_rel`
  - `enemy_set_hitbox`
  - `enemy_set_collision`
  - `ecl_time_sub`
  - `spellcard_start`
  - `set_int_difficulty`
  - `set_float_difficulty`
- [ ] 让补全、hover、参数提示优先使用人工参数名映射；没有命中时再回退到签名推断出的泛型参数名
- [ ] 为人工参数名映射设计独立模块，不把命名表硬编码进补全或 hover 提供器内部
