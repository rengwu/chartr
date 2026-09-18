import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BugIcon` instead.
 *
 * @example
 * ```svelte
 * <Bug color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Bug: Component<IconComponentProps, {}, "">;
type Bug = ReturnType<typeof Bug>;
export default Bug;
