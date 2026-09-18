import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SkipBackIcon` instead.
 *
 * @example
 * ```svelte
 * <SkipBack color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SkipBack: Component<IconComponentProps, {}, "">;
type SkipBack = ReturnType<typeof SkipBack>;
export default SkipBack;
