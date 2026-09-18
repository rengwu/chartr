import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `VinylRecordIcon` instead.
 *
 * @example
 * ```svelte
 * <VinylRecord color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const VinylRecord: Component<IconComponentProps, {}, "">;
type VinylRecord = ReturnType<typeof VinylRecord>;
export default VinylRecord;
