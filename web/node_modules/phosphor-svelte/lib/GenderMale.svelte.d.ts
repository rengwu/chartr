import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GenderMaleIcon` instead.
 *
 * @example
 * ```svelte
 * <GenderMale color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GenderMale: Component<IconComponentProps, {}, "">;
type GenderMale = ReturnType<typeof GenderMale>;
export default GenderMale;
