import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `IdentificationBadgeIcon` instead.
 *
 * @example
 * ```svelte
 * <IdentificationBadge color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const IdentificationBadge: Component<IconComponentProps, {}, "">;
type IdentificationBadge = ReturnType<typeof IdentificationBadge>;
export default IdentificationBadge;
