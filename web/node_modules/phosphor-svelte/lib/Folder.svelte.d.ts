import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FolderIcon` instead.
 *
 * @example
 * ```svelte
 * <Folder color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Folder: Component<IconComponentProps, {}, "">;
type Folder = ReturnType<typeof Folder>;
export default Folder;
