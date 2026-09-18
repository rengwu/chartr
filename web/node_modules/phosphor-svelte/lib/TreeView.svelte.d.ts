import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TreeViewIcon` instead.
 *
 * @example
 * ```svelte
 * <TreeView color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TreeView: Component<IconComponentProps, {}, "">;
type TreeView = ReturnType<typeof TreeView>;
export default TreeView;
