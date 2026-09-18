import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BaseballIcon` instead.
 *
 * @example
 * ```svelte
 * <Baseball color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Baseball: Component<IconComponentProps, {}, "">;
type Baseball = ReturnType<typeof Baseball>;
export default Baseball;
