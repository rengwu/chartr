import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CopyIcon` instead.
 *
 * @example
 * ```svelte
 * <Copy color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Copy: Component<IconComponentProps, {}, "">;
type Copy = ReturnType<typeof Copy>;
export default Copy;
