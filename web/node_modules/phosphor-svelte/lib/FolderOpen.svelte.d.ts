import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FolderOpenIcon` instead.
 *
 * @example
 * ```svelte
 * <FolderOpen color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FolderOpen: Component<IconComponentProps, {}, "">;
type FolderOpen = ReturnType<typeof FolderOpen>;
export default FolderOpen;
