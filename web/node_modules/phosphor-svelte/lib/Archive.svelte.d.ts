import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArchiveIcon` instead.
 *
 * @example
 * ```svelte
 * <Archive color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Archive: Component<IconComponentProps, {}, "">;
type Archive = ReturnType<typeof Archive>;
export default Archive;
