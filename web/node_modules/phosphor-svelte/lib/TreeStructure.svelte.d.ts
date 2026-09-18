import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TreeStructureIcon` instead.
 *
 * @example
 * ```svelte
 * <TreeStructure color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TreeStructure: Component<IconComponentProps, {}, "">;
type TreeStructure = ReturnType<typeof TreeStructure>;
export default TreeStructure;
