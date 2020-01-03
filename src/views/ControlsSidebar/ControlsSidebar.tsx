import React, { FunctionComponent, useEffect, useReducer, useState } from 'react';
import getUuid from 'uuid/v1';
import { connect } from 'react-redux';
import { useDebounce } from 'use-debounce';
import { Dispatch } from 'redux';
import usePrevious from 'use-previous';
import compact from 'lodash-es/compact';
import get from 'lodash-es/get';
import map from 'lodash-es/map';
import throttle from '@jcoreio/async-throttle';

import { applyModifications, saveImage, openImage } from '../../services/interop-electron';
import Button from '../../components/Button/Button';
import { AppState } from '../../store/types';
import { selectModifiedImage, selectOriginalImage } from '../../store/selectors';
import { setModificationInProgressAction } from '../../store/actions';
import { Select, SelectOption } from '../../components/Select/Select';
import { format } from '../../helpers/format';
import { ImageModificationInfo } from '../../shared/types';
import mods, { ModificationComponent } from '../../shared/modifications/components';

import './ControlsSidebar.scss';
import { fromPairs } from 'lodash-es';

interface ControlsSidebarProps {
  originalImageUrl: string | null;
  modifiedImageUrl: string | null;
  setModificationInProgress: (inProgress: boolean) => void;
}

interface ActiveModifier {
  name: string;
  title: string;
  info: ImageModificationInfo | null;
  isOpen: boolean;
  key: string;
}

const ControlsSidebar: FunctionComponent<ControlsSidebarProps> = ({
                                                                    originalImageUrl,
                                                                    modifiedImageUrl,
                                                                    setModificationInProgress
                                                                  }) => {
  const [selectedOption, setSelectedOption] = useState<string>('');

  const IMAGE_MODIFIER_OPTIONS: SelectOption[] = [
    {
      label: "Kies een bewerking",
      value: "",
      disabled: true
    },
    ...map(mods, (mod: ModificationComponent, modName: string) => {
      return {
        label: mod.label,
        value: modName
      }
    })
  ];

  const IMAGE_MODIFIER_TITLES: { [modifierName: string]: string } = fromPairs(map(mods, (mod: ModificationComponent, modName: string) => {
    return [modName, mod.title];
  }));

  type ActiveModifierAction =
      | { type: 'ADD', name: string }
      | { type: 'UPDATE', info: ImageModificationInfo, index: number }
      | { type: 'TOGGLE', index: number }
      | { type: 'DELETE', index: number }

  const getModifierTitle = (activeModifiers: ActiveModifier[], modifierName: string): string => {
    return format(IMAGE_MODIFIER_TITLES[modifierName], activeModifiers.filter(mod => mod.name === modifierName).length + 1);
  };

  function activeModifiersReducer(activeModifiers: ActiveModifier[], action: ActiveModifierAction): ActiveModifier[] {
    let modifiers: ActiveModifier[] | undefined;
    let key = getUuid();
    switch (action.type) {
      case 'ADD':
        const title: string = getModifierTitle(activeModifiers, action.name);
        const newModifier = {
          title,
          info: null,
          name: action.name,
          isOpen: true,
          key
        };
        return [
          ...activeModifiers.map(mod => {
            mod.isOpen = false; // Close all previous modifiers when adding a new one
            return mod;
          }),
          newModifier
        ];

      case 'UPDATE':
        modifiers = [...activeModifiers];
        modifiers[action.index] = {
          ...modifiers[action.index],
          info: action.info
        };
        return modifiers;

      case 'TOGGLE':
        const originalIsOpen: boolean = get(activeModifiers, `[${action.index}].isOpen`, false);
        modifiers = [
          ...activeModifiers.map(mod => {
            mod.isOpen = false; // Close all previous modifiers when adding a new one
            return mod;
          })
        ];
        modifiers[action.index] = {
          ...modifiers[action.index],
          isOpen: !originalIsOpen
        };
        return modifiers;

      case 'DELETE':
        modifiers = [
          ...activeModifiers
        ];
        modifiers.splice(action.index, 1);
        return modifiers;
    }
  }

  const [activeModifiers, modifyActiveModifiers] = useReducer<React.Reducer<ActiveModifier[], ActiveModifierAction>>(activeModifiersReducer, []);
  const [debouncedActiveModifiers] = useDebounce(activeModifiers, 500);
  const previousActiveModifiers = usePrevious<ActiveModifier[] | null>(activeModifiers);
  const applyModificationsThrottled = throttle((modifications: ImageModificationInfo[]) => {
    applyModifications(modifications);
  }, 0);

  useEffect(() => {
    const modifications: ImageModificationInfo[] = compact(debouncedActiveModifiers.map((m: ActiveModifier) => m.info));

    // check previous state to see if we need to run the applyModifications function because all modifiers were removed
    if (modifications && (modifications.length || (previousActiveModifiers && previousActiveModifiers.length !== modifications.length))) {
      setModificationInProgress(true);
      applyModificationsThrottled(modifications); // Do not apply new modifications if the previous modifications are not done yet
    }
  }, [debouncedActiveModifiers]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleApplyModifier = (modifierInfo: ImageModificationInfo, index: number) => {
    modifyActiveModifiers({ type: 'UPDATE', info: modifierInfo, index });
  };

  const handleToggleModifier = (index: number) => {
    modifyActiveModifiers({ type: 'TOGGLE', index });
  };

  const handleDeleteModifier = (index: number) => {
    modifyActiveModifiers({ type: 'DELETE', index });
  };

  const handleModifierSelectionChanged = (name: string) => {
    modifyActiveModifiers({ type: 'ADD', name });
    setSelectedOption('');
  };

  return (
      <div className="m-controls-sidebar">
        <div className="m-button-wrapper">
          <Button onClick={() => {
            openImage({
              buttonLabel: "Openen",
              // defaultPath // TODO default to the users images folder
              filters: [
                {
                  name: 'Alle bestanden',
                  extensions: ['*']
                },
                {
                  name: 'Afbeeldingen',
                  extensions: ['jpg', 'png', 'gif', 'tif', 'bmp', 'jpeg', 'webp']
                }
              ],
              title: 'Selecteer de afbeelding die je wil bewerken',
              properties: [
                // 'multiSelections', // TODO add multi image support
                'openFile'
              ]
            });
          }} label="Afbeelding openen"/>
          {modifiedImageUrl &&
          <Button className="c-save-button" onClick={() => {
            saveImage({
              buttonLabel: "Opslaan",
              // defaultPath // TODO default to the users images folder
              filters: [
                {
                  name: 'Alle bestanden',
                  extensions: ['*']
                },
                {
                  name: 'Afbeeldingen',
                  extensions: ['jpg', 'png', 'gif', 'tif', 'bmp', 'jpeg', 'webp']
                }
              ],
              title: 'Kies de locatie en naam om je afbeelding op te slaan'
            });
          }} label="Afbeelding opslaan als"/>
          }
        </div>
        {originalImageUrl &&
        <>
          <div className="m-button-wrapper">
            <Select
              options={IMAGE_MODIFIER_OPTIONS}
              onChange={handleModifierSelectionChanged}
              value={selectedOption}
            />
          </div>
          <div className="m-modification-controls">
            {activeModifiers.map((m: ActiveModifier, index: number) => {
              if (!mods[m.name] || !mods[m.name].component) {
                console.error('Failed to find component by name in list of modifications', { modificationInfo: m });
                return null;
              }
              const Component = mods[m.name].component;
              return <Component
                  title={m.title}
                  onApply={(info: ImageModificationInfo) => handleApplyModifier(info, index)}
                  isOpen={m.isOpen}
                  onToggle={() => handleToggleModifier(index)}
                  onDelete={() => handleDeleteModifier(index)}
                  key={m.key}/>;
            })}
          </div>
        </>
        }
      </div>
  );
};

const mapStateToProps = (state: { app: AppState }) => ({
  originalImageUrl: selectOriginalImage(state),
  modifiedImageUrl: selectModifiedImage(state)
});

const mapDispatchToProps = (dispatch: Dispatch) => {
  return {
    setModificationInProgress: (inProgress: boolean) => dispatch(setModificationInProgressAction(inProgress) as any)
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(ControlsSidebar);
