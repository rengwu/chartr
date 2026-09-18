import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ThreadsLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <ThreadsLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ThreadsLogo: Component<IconComponentProps, {}, "">;
type ThreadsLogo = ReturnType<typeof ThreadsLogo>;
export default ThreadsLogo;
