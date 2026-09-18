import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FoldersIcon` instead.
 *
 * @example
 * ```svelte
 * <Folders color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Folders: Component<IconComponentProps, {}, "">;
type Folders = ReturnType<typeof Folders>;
export default Folders;
