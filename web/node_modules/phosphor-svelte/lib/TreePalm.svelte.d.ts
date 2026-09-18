import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TreePalmIcon` instead.
 *
 * @example
 * ```svelte
 * <TreePalm color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TreePalm: Component<IconComponentProps, {}, "">;
type TreePalm = ReturnType<typeof TreePalm>;
export default TreePalm;
