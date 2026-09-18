import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StarOfDavidIcon` instead.
 *
 * @example
 * ```svelte
 * <StarOfDavid color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const StarOfDavid: Component<IconComponentProps, {}, "">;
type StarOfDavid = ReturnType<typeof StarOfDavid>;
export default StarOfDavid;
