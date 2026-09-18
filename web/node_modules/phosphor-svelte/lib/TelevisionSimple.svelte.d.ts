import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TelevisionSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <TelevisionSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TelevisionSimple: Component<IconComponentProps, {}, "">;
type TelevisionSimple = ReturnType<typeof TelevisionSimple>;
export default TelevisionSimple;
