import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <FolderSimplePlusIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FolderSimplePlusIcon: Component<IconComponentProps, {}, "">;
type FolderSimplePlusIcon = ReturnType<typeof FolderSimplePlusIcon>;
export default FolderSimplePlusIcon;
