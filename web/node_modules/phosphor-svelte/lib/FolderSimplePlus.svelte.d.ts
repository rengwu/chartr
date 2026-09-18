import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FolderSimplePlusIcon` instead.
 *
 * @example
 * ```svelte
 * <FolderSimplePlus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FolderSimplePlus: Component<IconComponentProps, {}, "">;
type FolderSimplePlus = ReturnType<typeof FolderSimplePlus>;
export default FolderSimplePlus;
