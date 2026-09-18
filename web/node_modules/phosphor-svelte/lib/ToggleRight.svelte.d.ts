import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ToggleRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ToggleRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ToggleRight: Component<IconComponentProps, {}, "">;
type ToggleRight = ReturnType<typeof ToggleRight>;
export default ToggleRight;
