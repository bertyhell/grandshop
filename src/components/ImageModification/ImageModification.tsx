import React, { FunctionComponent, ReactNode } from 'react';
import classnames from 'classnames';

import './ImageModification.scss';

interface ImageModificationProps {
  className?: string;
  label?: string;
  children: ReactNode;
}

const ImageModification: FunctionComponent<ImageModificationProps> = ({
                                                                        className,
                                                                        label,
                                                                        children
                                                                      }) => {
  return (
      <div
          className={classnames(className, 'o-image-modification')}
      >
        <h2 className="o-image-modification-label">{label}</h2>
        <div className="o-image-modification-controls">
          {children}
        </div>
      </div>
  );
};

export default ImageModification;
