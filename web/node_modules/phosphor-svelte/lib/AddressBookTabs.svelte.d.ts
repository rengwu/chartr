import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AddressBookTabsIcon` instead.
 *
 * @example
 * ```svelte
 * <AddressBookTabs color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AddressBookTabs: Component<IconComponentProps, {}, "">;
type AddressBookTabs = ReturnType<typeof AddressBookTabs>;
export default AddressBookTabs;
