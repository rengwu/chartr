import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `OptionIcon` instead.
 *
 * @example
 * ```svelte
 * <Option color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Option: Component<IconComponentProps, {}, "">;
type Option = ReturnType<typeof Option>;
export default Option;
