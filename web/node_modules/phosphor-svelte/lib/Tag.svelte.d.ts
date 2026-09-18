import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TagIcon` instead.
 *
 * @example
 * ```svelte
 * <Tag color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Tag: Component<IconComponentProps, {}, "">;
type Tag = ReturnType<typeof Tag>;
export default Tag;
