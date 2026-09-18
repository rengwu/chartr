import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileLockIcon` instead.
 *
 * @example
 * ```svelte
 * <FileLock color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileLock: Component<IconComponentProps, {}, "">;
type FileLock = ReturnType<typeof FileLock>;
export default FileLock;
