import { FunctionComponent } from 'react';
import ReactSelect, { ActionMeta } from 'react-select';
import clsx from 'clsx';

import './Select.scss';

export interface SelectOption<T = string> {
	value: T;
	label: string;
	disabled?: boolean;
}

export interface SelectProps {
	className?: string;
	options: SelectOption[];
	id?: string;
	disabled?: boolean;
	loading?: boolean;
	clearable?: boolean;
	value: string | null;
	placeholder?: string;
	onChange?: (value: string) => void;
}

export const Select: FunctionComponent<SelectProps> = ({
	className,
	options,
	id,
	disabled = false,
	loading = false,
	clearable = false,
	placeholder,
	value,
	onChange = () => {},
}) => {
	function onValueChange(
		changedValue: SelectOption<string>,
		actionMeta: ActionMeta<SelectOption>
	) {
		if (actionMeta.action !== 'create-option' && changedValue) {
			onChange(changedValue.value);
		}
	}

	return (
		<ReactSelect
			className={clsx('c-select', className)}
			classNamePrefix="c-select"
			id={id}
			options={options}
			value={options.find((option) => option.value === value) || null}
			isMulti={false}
			isDisabled={disabled}
			isLoading={loading}
			isClearable={clearable}
			isOptionDisabled={(option) => !!option.disabled}
			placeholder={placeholder}
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			onChange={onValueChange as any}
		/>
	);
};
