import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TagSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <TagSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TagSimple: Component<IconComponentProps, {}, "">;
type TagSimple = ReturnType<typeof TagSimple>;
export default TagSimple;
