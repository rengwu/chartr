import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <FolderSimpleDashedIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FolderSimpleDashedIcon: Component<IconComponentProps, {}, "">;
type FolderSimpleDashedIcon = ReturnType<typeof FolderSimpleDashedIcon>;
export default FolderSimpleDashedIcon;
