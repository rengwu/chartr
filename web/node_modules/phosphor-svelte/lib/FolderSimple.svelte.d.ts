import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FolderSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <FolderSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FolderSimple: Component<IconComponentProps, {}, "">;
type FolderSimple = ReturnType<typeof FolderSimple>;
export default FolderSimple;
