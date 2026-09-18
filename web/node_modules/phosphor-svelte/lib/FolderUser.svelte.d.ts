import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FolderUserIcon` instead.
 *
 * @example
 * ```svelte
 * <FolderUser color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FolderUser: Component<IconComponentProps, {}, "">;
type FolderUser = ReturnType<typeof FolderUser>;
export default FolderUser;
