import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RectangleDashedIcon` instead.
 *
 * @example
 * ```svelte
 * <RectangleDashed color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const RectangleDashed: Component<IconComponentProps, {}, "">;
type RectangleDashed = ReturnType<typeof RectangleDashed>;
export default RectangleDashed;
