import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FolderLockIcon` instead.
 *
 * @example
 * ```svelte
 * <FolderLock color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FolderLock: Component<IconComponentProps, {}, "">;
type FolderLock = ReturnType<typeof FolderLock>;
export default FolderLock;
