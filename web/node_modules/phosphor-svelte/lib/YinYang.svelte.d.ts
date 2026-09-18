import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `YinYangIcon` instead.
 *
 * @example
 * ```svelte
 * <YinYang color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const YinYang: Component<IconComponentProps, {}, "">;
type YinYang = ReturnType<typeof YinYang>;
export default YinYang;
