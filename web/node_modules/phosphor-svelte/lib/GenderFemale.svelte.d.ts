import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GenderFemaleIcon` instead.
 *
 * @example
 * ```svelte
 * <GenderFemale color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GenderFemale: Component<IconComponentProps, {}, "">;
type GenderFemale = ReturnType<typeof GenderFemale>;
export default GenderFemale;
