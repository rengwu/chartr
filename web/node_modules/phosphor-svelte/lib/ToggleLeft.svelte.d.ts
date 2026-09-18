import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ToggleLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ToggleLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ToggleLeft: Component<IconComponentProps, {}, "">;
type ToggleLeft = ReturnType<typeof ToggleLeft>;
export default ToggleLeft;
