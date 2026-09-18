import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DnaIcon` instead.
 *
 * @example
 * ```svelte
 * <Dna color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Dna: Component<IconComponentProps, {}, "">;
type Dna = ReturnType<typeof Dna>;
export default Dna;
