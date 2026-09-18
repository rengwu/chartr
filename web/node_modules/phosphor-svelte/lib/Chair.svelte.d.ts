import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChairIcon` instead.
 *
 * @example
 * ```svelte
 * <Chair color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Chair: Component<IconComponentProps, {}, "">;
type Chair = ReturnType<typeof Chair>;
export default Chair;
