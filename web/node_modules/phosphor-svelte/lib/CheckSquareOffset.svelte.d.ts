import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CheckSquareOffsetIcon` instead.
 *
 * @example
 * ```svelte
 * <CheckSquareOffset color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CheckSquareOffset: Component<IconComponentProps, {}, "">;
type CheckSquareOffset = ReturnType<typeof CheckSquareOffset>;
export default CheckSquareOffset;
