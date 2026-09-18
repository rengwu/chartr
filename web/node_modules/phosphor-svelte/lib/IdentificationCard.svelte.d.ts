import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `IdentificationCardIcon` instead.
 *
 * @example
 * ```svelte
 * <IdentificationCard color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const IdentificationCard: Component<IconComponentProps, {}, "">;
type IdentificationCard = ReturnType<typeof IdentificationCard>;
export default IdentificationCard;
