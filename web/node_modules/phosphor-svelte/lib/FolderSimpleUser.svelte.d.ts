import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FolderSimpleUserIcon` instead.
 *
 * @example
 * ```svelte
 * <FolderSimpleUser color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FolderSimpleUser: Component<IconComponentProps, {}, "">;
type FolderSimpleUser = ReturnType<typeof FolderSimpleUser>;
export default FolderSimpleUser;
