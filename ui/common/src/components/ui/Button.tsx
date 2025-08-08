import React from 'react';
import { Button as AntButton, ButtonProps } from 'antd';
import styled from 'styled-components';

const StyledButton = styled(AntButton)<{ variant?: 'primary' | 'secondary' | 'danger' | 'ghost' }>`
  border-radius: 8px;
  font-weight: 500;
  transition: all 0.3s ease;
  
  ${props => props.variant === 'primary' && `
    background: linear-gradient(135deg, #00d4ff, #0099cc);
    border: none;
    
    &:hover {
      background: linear-gradient(135deg, #00b8e6, #0088bb);
      transform: translateY(-1px);
      box-shadow: 0 4px 12px rgba(0, 212, 255, 0.3);
    }
  `}
  
  ${props => props.variant === 'secondary' && `
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: rgba(255, 255, 255, 0.8);
    
    &:hover {
      background: rgba(255, 255, 255, 0.15);
      border-color: #00d4ff;
      color: #00d4ff;
    }
  `}
  
  ${props => props.variant === 'ghost' && `
    background: transparent;
    border: 1px solid #00d4ff;
    color: #00d4ff;
    
    &:hover {
      background: rgba(0, 212, 255, 0.1);
    }
  `}
`;

interface CustomButtonProps extends ButtonProps {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
}

const Button: React.FC<CustomButtonProps> = ({ variant = 'primary', ...props }) => {
  return <StyledButton variant={variant} {...props} />;
};

export default Button;