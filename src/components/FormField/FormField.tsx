import React, { FunctionComponent, ReactNode } from 'react';
import classnames from 'classnames';

import './FormField.scss';

interface FormFieldProps {
  className?: string;
  label?: string;
  labelFor?: string;
  required?: boolean;
  error?: string | string[];
  children: ReactNode;
}

const FormField: FunctionComponent<FormFieldProps> = ({
                                                        className,
                                                        label,
                                                        labelFor,
                                                        required = false,
                                                        error,
                                                        children,
                                                }) => {
  const errorArray = typeof error === 'string' ? [error] : error;

  return (
      <div
          className={classnames(className, 'o-form-field', {
            'o-form-field-error': error,
          })}
      >
        {label && (
            <label className="o-form-field-label" htmlFor={labelFor}>
              {label}
              {required ? (
                  <abbr className="required" title="Verplicht veld">
                    *
                  </abbr>
              ) : null}
            </label>
        )}
        <div className="o-form-field-controls">
          {children}
          {errorArray &&
          errorArray.length &&
          errorArray.map((err: string, index: number) => (
              <div key={`${err}-${index}`} className="c-form-help-text c-form-help-text-error">
                {err}
              </div>
          ))}
        </div>
      </div>
  );
};

export default FormField;
