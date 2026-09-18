import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FalloutShelterIcon` instead.
 *
 * @example
 * ```svelte
 * <FalloutShelter color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FalloutShelter: Component<IconComponentProps, {}, "">;
type FalloutShelter = ReturnType<typeof FalloutShelter>;
export default FalloutShelter;
