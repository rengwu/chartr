import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FolderPlusIcon` instead.
 *
 * @example
 * ```svelte
 * <FolderPlus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FolderPlus: Component<IconComponentProps, {}, "">;
type FolderPlus = ReturnType<typeof FolderPlus>;
export default FolderPlus;
