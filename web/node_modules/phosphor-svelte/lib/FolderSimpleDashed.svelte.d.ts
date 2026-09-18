import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FolderSimpleDashedIcon` instead.
 *
 * @example
 * ```svelte
 * <FolderSimpleDashed color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FolderSimpleDashed: Component<IconComponentProps, {}, "">;
type FolderSimpleDashed = ReturnType<typeof FolderSimpleDashed>;
export default FolderSimpleDashed;
